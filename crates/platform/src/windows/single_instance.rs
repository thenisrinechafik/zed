use std::ffi::c_void;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use log::{debug, warn};
use release_channel::app_identifier;
use uuid::Uuid;
use windows::core::{Error as WinError, HSTRING};
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, GetLastError, HANDLE};
use windows::Win32::Security::{ConvertStringSecurityDescriptorToSecurityDescriptorW, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, SDDL_REVISION_1};
use windows::Win32::Storage::FileSystem::{CreateFileW, CreateNamedPipeW, ReadFile, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE, PIPE_ACCESS_INBOUND, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES};
use windows::Win32::System::Memory::LocalFree;
use windows::Win32::System::Pipes::{ConnectNamedPipe, DisconnectNamedPipe, FlushFileBuffers, SetNamedPipeHandleState, WaitNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_WAIT};
use windows::Win32::System::Threading::CreateMutexW;

/// Upper bound used when attempting to reach the running instance.
const IPC_CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const IPC_RETRY_DELAY: Duration = Duration::from_millis(20);
const PIPE_BUFFER_SIZE: u32 = 2048;

/// Guard returned by [`acquire_lock`] that manages the single-instance mutex.
pub struct InstanceGuard {
    mutex: HANDLE,
    pipe_name: HSTRING,
    is_primary: bool,
}

impl InstanceGuard {
    /// Returns `true` if this process is the primary instance.
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    /// Returns the name of the named pipe used to communicate with the launcher.
    pub fn pipe_name(&self) -> &HSTRING {
        &self.pipe_name
    }

    /// Spawns a background listener that invokes `handler` for each payload received from
    /// secondary instances.
    pub fn spawn_listener<F>(&self, handler: F) -> Result<thread::JoinHandle<()>>
    where
        F: Fn(String) + Send + 'static,
    {
        let pipe_name = self.pipe_name.clone();
        let debug_enabled = ipc_debug_enabled();

        let handle = thread::Builder::new()
            .name("zed-ipc-listener".into())
            .spawn(move || loop {
                match create_pipe(&pipe_name) {
                    Ok(pipe) => {
                        if let Err(err) = wait_for_client(pipe) {
                            warn!("windows ipc: failed waiting for client: {err:?}");
                            unsafe {
                                CloseHandle(pipe);
                            }
                            thread::sleep(IPC_RETRY_DELAY);
                            continue;
                        }

                        match read_message(pipe) {
                            Ok(message) => {
                                if debug_enabled {
                                    debug!("windows ipc: received payload `{message}`");
                                }
                                handler(message);
                            }
                            Err(err) => {
                                warn!("windows ipc: failed reading payload: {err:?}");
                            }
                        }

                        unsafe {
                            let _ = FlushFileBuffers(pipe);
                            let _ = DisconnectNamedPipe(pipe);
                            CloseHandle(pipe);
                        }
                    }
                    Err(err) => {
                        warn!("windows ipc: unable to create named pipe: {err:?}");
                        thread::sleep(IPC_RETRY_DELAY);
                    }
                }
            })?;

        Ok(handle)
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.mutex.is_invalid() {
                let _ = CloseHandle(self.mutex);
            }
        }
    }
}

/// Attempts to acquire the single-instance mutex.
pub fn acquire_lock() -> Result<InstanceGuard> {
    let (mutex_name, pipe_name) = ipc_identity();
    let mutex = with_permissive_security(|security| unsafe {
        CreateMutexW(Some(security), false, &mutex_name)
    })
    .context("CreateMutexW")?;
    let is_primary = unsafe { GetLastError() != ERROR_ALREADY_EXISTS };

    if ipc_debug_enabled() {
        debug!(
            "windows ipc: {} primary instance using pipe {:?}",
            if is_primary { "running as" } else { "connected to" },
            pipe_name
        );
    }

    Ok(InstanceGuard {
        mutex,
        pipe_name,
        is_primary,
    })
}

/// Signals the running instance using the named pipe channel.
pub fn signal_running_instance(payload: &[u8]) -> Result<()> {
    let (_, pipe_name) = ipc_identity();
    let payload = ensure_payload(payload);
    let deadline = Instant::now() + IPC_CONNECT_TIMEOUT;

    loop {
        match unsafe {
            CreateFileW(
                &pipe_name,
                FILE_GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                windows::Win32::Storage::FileSystem::OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            )
        } {
            Ok(pipe) => {
                let mut written = 0u32;
                unsafe {
                    WriteFile(pipe, Some(&payload), Some(&mut written), None)
                        .context("WriteFile")?;
                    FlushFileBuffers(pipe).context("FlushFileBuffers")?;
                    CloseHandle(pipe);
                }
                if ipc_debug_enabled() {
                    debug!("windows ipc: forwarded payload to primary");
                }
                return Ok(());
            }
            Err(err) => {
                let last_error = unsafe { GetLastError() };
                if last_error == ERROR_PIPE_BUSY {
                    unsafe {
                        WaitNamedPipeW(&pipe_name, IPC_RETRY_DELAY.as_millis() as u32)
                            .ok()
                            .ok();
                    }
                } else if last_error == ERROR_FILE_NOT_FOUND {
                    // Stale mutex owner – the original process likely crashed. Give the caller
                    // a chance to become primary by returning an error.
                    return Err(err.into());
                }

                if Instant::now() >= deadline {
                    return Err(err.into());
                }

                thread::sleep(IPC_RETRY_DELAY);
            }
        }
    }
}

fn wait_for_client(pipe: HANDLE) -> Result<()> {
    unsafe {
        match ConnectNamedPipe(pipe, None) {
            Ok(()) => Ok(()),
            Err(err) => {
                let last_error = GetLastError();
                if last_error == ERROR_PIPE_CONNECTED {
                    Ok(())
                } else {
                    Err(err.into())
                }
            }
        }
    }
}

fn read_message(pipe: HANDLE) -> Result<String> {
    let mut buffer = vec![0u8; PIPE_BUFFER_SIZE as usize];
    let mut collected = Vec::with_capacity(buffer.len());

    loop {
        let mut bytes_read = 0u32;
        unsafe {
            ReadFile(pipe, Some(&mut buffer), Some(&mut bytes_read), None)
                .context("ReadFile")?;
        }

        if bytes_read == 0 {
            break;
        }

        collected.extend_from_slice(&buffer[..bytes_read as usize]);
        if bytes_read < PIPE_BUFFER_SIZE {
            break;
        }
    }

    // Trim trailing NUL bytes introduced by the writer for compatibility with existing logic.
    while collected.last().copied() == Some(0) {
        collected.pop();
    }

    Ok(String::from_utf8_lossy(&collected).into_owned())
}

fn create_pipe(name: &HSTRING) -> Result<HANDLE> {
    with_permissive_security(|security| unsafe {
        let pipe = CreateNamedPipeW(
            name,
            PIPE_ACCESS_INBOUND | PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            PIPE_BUFFER_SIZE,
            PIPE_BUFFER_SIZE,
            0,
            Some(security),
        );

        if pipe.is_invalid() {
            Err(WinError::from_win32().into())
        } else {
            // Ensure clients read messages rather than byte streams.
            SetNamedPipeHandleState(pipe, Some(&PIPE_READMODE_MESSAGE), None, None)
                .ok()
                .context("SetNamedPipeHandleState")?;
            Ok(pipe)
        }
    })
}

fn with_permissive_security<F, T>(f: F) -> Result<T>
where
    F: FnOnce(&SECURITY_ATTRIBUTES) -> Result<T>,
{
    unsafe {
        let descriptor_string = HSTRING::from("D:(A;;GA;;;WD)(A;;GA;;;AN)");
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            &descriptor_string,
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
        .ok()
        .context("ConvertStringSecurityDescriptorToSecurityDescriptorW")?;

        let mut attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0.cast::<c_void>(),
            bInheritHandle: 0,
        };

        let result = f(&attributes);
        let _ = LocalFree(descriptor.0.cast());
        result
    }
}

fn ensure_payload(payload: &[u8]) -> Vec<u8> {
    if payload.last().copied() == Some(0) {
        payload.to_vec()
    } else {
        let mut owned = payload.to_vec();
        owned.push(0);
        owned
    }
}

fn ipc_identity() -> (HSTRING, HSTRING) {
    let app_id = app_identifier();
    let namespace = Uuid::NAMESPACE_OID;
    let uuid = Uuid::new_v5(&namespace, app_id.as_bytes());
    let mutex_name = HSTRING::from(format!("Global\\\\Zed_{}", uuid));
    let pipe_name = HSTRING::from(format!("\\\\.\\pipe\\zed-launcher-{}", uuid));
    (mutex_name, pipe_name)
}

fn ipc_debug_enabled() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| std::env::var_os("ZED_IPC_DEBUG").is_some())
}
