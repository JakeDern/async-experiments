use std::io;
use std::os::fd::RawFd;

use crate::{sys, syscall};

pub struct Reactor {
    epoll_fd: RawFd,
}

impl Reactor {
    pub fn new() -> io::Result<Self> {
        return Ok(Self {
            epoll_fd: sys::epoll_create()?,
        });
    }

    pub fn register_interest(&self, fd: RawFd, interest: i32) -> io::Result<i32> {
        let mut event = libc::epoll_event {
            events: interest as u32,
            u64: fd as u64,
        };

        syscall!(epoll_ctl(
            self.epoll_fd,
            libc::EPOLL_CTL_ADD,
            fd,
            &mut event
        ))
    }

    pub fn unregister_interest(&self, fd: RawFd) -> io::Result<i32> {
        syscall!(epoll_ctl(
            self.epoll_fd,
            libc::EPOLL_CTL_DEL,
            fd,
            std::ptr::null_mut()
        ))
    }

    pub fn wait_for_events(
        &self,
        events: &mut [libc::epoll_event],
        timeout: i32,
    ) -> io::Result<usize> {
        syscall!(epoll_wait(
            self.epoll_fd,
            events.as_mut_ptr(),
            events.len() as i32,
            timeout
        ))
    }
}
