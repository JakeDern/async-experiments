use std::io;
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::prelude::RawFd;

use crate::sys;

pub struct TcpListener {
    fd: RawFd,
}

impl TcpListener {
    pub fn bind(addr: SocketAddr, backlog_size: i32) -> io::Result<Self> {
        let fd = sys::open_tcp_socket(addr)?;
        sys::bind_socket(fd, addr, true)?;
        sys::listen_socket(fd, backlog_size)?;
        Ok(Self { fd })
    }
}

impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        todo!()
    }
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.clone()
    }
}
