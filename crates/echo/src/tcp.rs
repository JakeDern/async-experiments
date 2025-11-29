use std::io;
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::prelude::RawFd;

use crate::{reactor, sys};

pub struct TcpListener {
    fd: RawFd,
}

impl TcpListener {
    pub fn bind(addr: SocketAddr, backlog_size: i32) -> io::Result<Self> {
        let fd = sys::open_tcp_socket(addr)?;
        sys::sock_bind(fd, addr, true)?;
        sys::sock_listen(fd, backlog_size)?;

        reactor::register_interest(fd, libc::EPOLLIN)?;
        Ok(Self { fd })
    }

    pub fn accept(&self) -> AcceptFuture {
        AcceptFuture { fd: self.fd }
    }
}

impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }
}

impl AsRawFd for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.clone()
    }
}

pub struct AcceptFuture {
    fd: RawFd,
}

impl Future for AcceptFuture {
    type Output = Result<(RawFd, SocketAddr), std::io::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match sys::sock_accept_nonblock(self.fd) {
            Ok((new_fd, addr)) => std::task::Poll::Ready(Ok((new_fd, addr))),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                let waker = ctx.waker().clone().into();
                reactor::register_wake(self.fd, waker).unwrap();
                std::task::Poll::Pending
            }
            Err(e) => Err(e).into(),
        }
    }
}
