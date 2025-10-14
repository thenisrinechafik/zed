use std::sync::Arc;

use anyhow::{Context, Result};
use cli::{ipc::IpcOneShotServer, CliRequest, CliResponse, IpcHandshake};
use parking_lot::Mutex;
use util::ResultExt;

use crate::{Args, OpenListener, RawOpenRequest};

#[cfg(all(feature = "win-ipc", target_os = "windows"))]
pub fn handle_single_instance(opener: OpenListener, args: &Args) -> bool {
    improved::handle_single_instance(opener, args)
}

#[cfg(not(all(feature = "win-ipc", target_os = "windows")))]
pub fn handle_single_instance(opener: OpenListener, args: &Args) -> bool {
    legacy::handle_single_instance(opener, args)
}

#[cfg(all(feature = "win-ipc", target_os = "windows"))]
mod improved {
    use super::*;
    use platform::windows::single_instance;

    pub(super) fn handle_single_instance(opener: OpenListener, args: &Args) -> bool {
        let guard = match single_instance::acquire_lock() {
            Ok(guard) => guard,
            Err(err) => {
                log::error!("failed to acquire Windows single-instance mutex: {err:#}");
                return super::legacy::handle_single_instance(opener, args);
            }
        };

        if guard.is_primary() {
            if let Err(err) = start_listener(&guard, opener) {
                log::error!("unable to start IPC listener: {err:#}");
            }
            // Leak the guard so the mutex stays held for the process lifetime.
            std::mem::forget(guard);
            true
        } else {
            drop(guard);
            if !args.foreground {
                super::send_args_to_instance(args).log_err();
            }
            false
        }
    }

    fn start_listener(guard: &single_instance::InstanceGuard, opener: OpenListener) -> Result<()> {
        let thread = guard.spawn_listener(move |payload| {
            opener.open(RawOpenRequest {
                urls: vec![payload],
                ..Default::default()
            })
        })?;
        drop(thread);
        Ok(())
    }
}

mod legacy {
    use super::*;
    use release_channel::app_identifier;
    use std::ffi::CStr;
    use util::ResultExt;
    use windows::{
        core::HSTRING,
        Win32::{
            Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GENERIC_WRITE, GetLastError, HANDLE},
            Storage::FileSystem::{
                CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING,
                PIPE_ACCESS_INBOUND, ReadFile, WriteFile,
            },
            System::{
                Pipes::{
                    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE,
                    PIPE_TYPE_MESSAGE, PIPE_WAIT,
                },
                Threading::CreateMutexW,
            },
        },
    };

    pub(super) fn handle_single_instance(opener: OpenListener, args: &Args) -> bool {
        let is_first_instance = is_first_instance();
        if is_first_instance {
            spawn_listener(opener);
        } else if !args.foreground {
            super::send_args_to_instance(args).log_err();
        }

        is_first_instance
    }

    fn spawn_listener(opener: OpenListener) {
        std::thread::Builder::new()
            .name("EnsureSingleton".to_owned())
            .spawn(move || {
                with_pipe(|url| {
                    opener.open(RawOpenRequest {
                        urls: vec![url],
                        ..Default::default()
                    })
                })
            })
            .unwrap();
    }

    fn is_first_instance() -> bool {
        unsafe {
            CreateMutexW(
                None,
                false,
                &HSTRING::from(format!("{}-Instance-Mutex", app_identifier())),
            )
            .expect("Unable to create instance mutex.")
        };
        unsafe { GetLastError() != ERROR_ALREADY_EXISTS }
    }

    fn with_pipe(f: impl Fn(String)) {
        let pipe = unsafe {
            CreateNamedPipeW(
                &HSTRING::from(format!("\\\\.\\pipe\\{}-Named-Pipe", app_identifier())),
                PIPE_ACCESS_INBOUND,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1,
                128,
                128,
                0,
                None,
            )
        };
        if pipe.is_invalid() {
            log::error!("Failed to create named pipe: {:?}", unsafe {
                GetLastError()
            });
            return;
        }

        loop {
            if let Some(message) = retrieve_message_from_pipe(pipe)
                .context("Failed to read from named pipe")
                .log_err()
            {
                f(message);
            }
        }
    }

    fn retrieve_message_from_pipe(pipe: HANDLE) -> anyhow::Result<String> {
        unsafe { ConnectNamedPipe(pipe, None)? };
        let message = retrieve_message_from_pipe_inner(pipe);
        unsafe { DisconnectNamedPipe(pipe).log_err() };
        message
    }

    fn retrieve_message_from_pipe_inner(pipe: HANDLE) -> anyhow::Result<String> {
        let mut buffer = [0u8; 128];
        unsafe {
            ReadFile(pipe, Some(&mut buffer), None, None)?;
        }
        let message = CStr::from_bytes_until_nul(&buffer)?;
        Ok(message.to_string_lossy().into_owned())
    }
}

pub fn send_args_to_instance(args: &Args) -> anyhow::Result<()> {
    if let Some(dock_menu_action_idx) = args.dock_action {
        let url = format!("zed-dock-action://{}", dock_menu_action_idx);
        return write_message_to_instance_pipe(url.as_bytes());
    }

    let (server, server_name) =
        IpcOneShotServer::<IpcHandshake>::new().context("Handshake before Zed spawn")?;
    let url = format!("zed-cli://{server_name}");

    let request = {
        let mut paths = vec![];
        let mut urls = vec![];
        let mut diff_paths = vec![];
        for path in args.paths_or_urls.iter() {
            match std::fs::canonicalize(&path) {
                Ok(path) => paths.push(path.to_string_lossy().into_owned()),
                Err(error) => {
                    if path.starts_with("zed://")
                        || path.starts_with("http://")
                        || path.starts_with("https://")
                        || path.starts_with("file://")
                        || path.starts_with("ssh://")
                    {
                        urls.push(path.clone());
                    } else {
                        log::error!("error parsing path argument: {}", error);
                    }
                }
            }
        }

        for path in args.diff.chunks(2) {
            let old = std::fs::canonicalize(&path[0]).log_err();
            let new = std::fs::canonicalize(&path[1]).log_err();
            if let Some((old, new)) = old.zip(new) {
                diff_paths.push([
                    old.to_string_lossy().into_owned(),
                    new.to_string_lossy().into_owned(),
                ]);
            }
        }

        CliRequest::Open {
            paths,
            urls,
            diff_paths,
            wait: false,
            wsl: args.wsl.clone(),
            open_new_workspace: None,
            env: None,
            user_data_dir: args.user_data_dir.clone(),
        }
    };

    let exit_status = Arc::new(Mutex::new(None));
    let sender: std::thread::JoinHandle<anyhow::Result<()>> = std::thread::Builder::new()
        .name("CliReceiver".to_owned())
        .spawn({
            let exit_status = exit_status.clone();
            move || {
                let (_, handshake) = server.accept().context("Handshake after Zed spawn")?;
                let (tx, rx) = (handshake.requests, handshake.responses);

                tx.send(request)?;

                while let Ok(response) = rx.recv() {
                    match response {
                        CliResponse::Ping => {}
                        CliResponse::Stdout { message } => log::info!("{message}"),
                        CliResponse::Stderr { message } => log::error!("{message}"),
                        CliResponse::Exit { status } => {
                            exit_status.lock().replace(status);
                            return Ok(());
                        }
                    }
                }
                Ok(())
            }
        })
        .unwrap();

    write_message_to_instance_pipe(url.as_bytes())?;
    sender.join().unwrap()?;
    if let Some(exit_status) = exit_status.lock().take() {
        std::process::exit(exit_status);
    }
    Ok(())
}

#[cfg(all(feature = "win-ipc", target_os = "windows"))]
fn write_message_to_instance_pipe(message: &[u8]) -> anyhow::Result<()> {
    use platform::windows::single_instance::signal_running_instance;
    signal_running_instance(message).context("signal primary instance")
}

#[cfg(not(all(feature = "win-ipc", target_os = "windows")))]
fn write_message_to_instance_pipe(message: &[u8]) -> anyhow::Result<()> {
    use release_channel::app_identifier;
    use windows::{
        core::HSTRING,
        Win32::{
            Foundation::{CloseHandle, GENERIC_WRITE},
            Storage::FileSystem::{CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING, WriteFile},
        },
    };

    unsafe {
        let pipe = CreateFileW(
            &HSTRING::from(format!("\\\\.\\pipe\\{}-Named-Pipe", app_identifier())),
            GENERIC_WRITE.0,
            FILE_SHARE_MODE::default(),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES::default(),
            None,
        )?;
        WriteFile(pipe, Some(message), None, None)?;
        CloseHandle(pipe)?;
    }
    Ok(())
}
