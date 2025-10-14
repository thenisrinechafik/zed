#![cfg(all(target_os = "windows", feature = "win-askpass-pipe"))]

use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};

use anyhow::{Context, Result};
use smol::io::{AsyncRead, AsyncWrite};
use smol::Async;
use uuid::Uuid;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_EXISTING};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, SetNamedPipeHandleState, PIPE_ACCESS_DUPLEX, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

pub struct NamedPipeListener {
    name: String,
}

impl NamedPipeListener {
    pub fn bind(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        ensure_pipe_prefix(&name)?;
        Ok(Self { name })
    }

    pub async fn accept(&self) -> Result<NamedPipeStream> {
        let name = self.name.clone();
        smol::unblock(move || create_server(&name)).await
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct NamedPipeStream(Async<std::fs::File>);

impl NamedPipeStream {
    pub async fn connect(name: &str) -> Result<Self> {
        let file = smol::unblock({
            let name = name.to_owned();
            move || connect_blocking(&name)
        })
        .await?;
        Ok(Self(Async::new(file)?))
    }
}

impl AsRawHandle for NamedPipeStream {
    fn as_raw_handle(&self) -> RawHandle {
        self.0.get_ref().as_raw_handle()
    }
}

impl AsyncRead for NamedPipeStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for NamedPipeStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        std::pin::Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_close(cx)
    }
}

pub fn unique_pipe_name() -> String {
    format!(r"\\\\.\\pipe\\zed-askpass-{}", Uuid::new_v4())
}

fn create_server(name: &str) -> Result<NamedPipeStream> {
    ensure_pipe_prefix(name)?;
    let name_w = wide_string(name);
    unsafe {
        let handle = CreateNamedPipeW(
            PCWSTR(name_w.as_ptr()),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_UNLIMITED_INSTANCES,
            4096,
            4096,
            0,
            None,
        );

        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error()).context("CreateNamedPipeW");
        }

        let connected = ConnectNamedPipe(handle, None);
        if connected.as_bool() {
            // ok
        } else {
            let err = io::Error::last_os_error();
            if err.raw_os_error() != Some(ERROR_PIPE_CONNECTED.0 as i32) {
                return Err(err).context("ConnectNamedPipe");
            }
        }

        let mut mode = (PIPE_READMODE_BYTE | PIPE_WAIT).0;
        if SetNamedPipeHandleState(handle, Some(&mut mode), None, None).as_bool() == false {
            return Err(io::Error::last_os_error()).context("SetNamedPipeHandleState");
        }

        let file = std::fs::File::from_raw_handle(handle_to_raw(handle));
        Ok(NamedPipeStream(Async::new(file)?))
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn ensure_pipe_prefix(name: &str) -> Result<()> {
    anyhow::ensure!(
        name.starts_with(r"\\\\.\\pipe\\"),
        "named pipe must start with \\..\\pipe\\"
    );
    Ok(())
}

pub fn connect_blocking(name: &str) -> Result<std::fs::File> {
    ensure_pipe_prefix(name)?;
    let name_w = wide_string(name);
    let handle = unsafe {
        CreateFileW(
            PCWSTR(name_w.as_ptr()),
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    };

    if handle.is_invalid() {
        return Err(io::Error::last_os_error()).context("opening named pipe client");
    }

    Ok(unsafe { std::fs::File::from_raw_handle(handle_to_raw(handle)) })
}

fn handle_to_raw(handle: windows::Win32::Foundation::HANDLE) -> RawHandle {
    handle.0 as isize as RawHandle
}
