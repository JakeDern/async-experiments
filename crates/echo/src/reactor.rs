use std::collections::HashMap;
use std::io;
use std::os::fd::RawFd;
use std::task::Waker;

use crate::{REACTOR, sys, syscall};

pub fn wait_and_wake(events: &mut [libc::epoll_event], timeout: i32) -> io::Result<i32> {
    REACTOR.with_borrow(|reactor| {
        let react = reactor
            .as_ref()
            .expect("Reactor not started on this thread");
        react.wait_for_events(events, timeout)
    })
}

pub fn register_interest(fd: RawFd, interest: i32) -> io::Result<i32> {
    REACTOR.with_borrow_mut(|reactor| {
        let react = reactor
            .as_mut()
            .expect("Reactor not started on this thread");
        react.register_interest(fd, interest)
    })
}

pub fn register_wake(fd: RawFd, waker: Waker) -> io::Result<()> {
    REACTOR.with_borrow_mut(|reactor| {
        let react = reactor
            .as_mut()
            .expect("Reactor not started on this thread");
        react.interest_set.insert(fd, Some(waker));
        Ok(())
    })
}

pub struct Reactor {
    epoll_fd: RawFd,
    interest_set: HashMap<RawFd, Option<Waker>>,
}

impl Reactor {
    pub fn new() -> io::Result<Self> {
        return Ok(Self {
            epoll_fd: sys::epoll_create()?,
            interest_set: HashMap::new(),
        });
    }

    pub fn register_interest(&mut self, fd: RawFd, interest: i32) -> io::Result<i32> {
        let mut event = libc::epoll_event {
            events: interest as u32,
            u64: fd as u64,
        };

        let result = syscall!(epoll_ctl(
            self.epoll_fd,
            libc::EPOLL_CTL_ADD,
            fd,
            &mut event
        ))?;

        self.interest_set.insert(fd, None);
        Ok(result)
    }

    pub fn register_wake(&mut self, fd: RawFd, waker: Waker) {
        // TODO: Is it realistic for multiple wakers to be registered for the same fd?
        self.interest_set.insert(fd, Some(waker));
    }

    // TODO: Make sure we do this on drop of any io stuff
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
    ) -> io::Result<i32> {
        let res = syscall!(epoll_wait(
            self.epoll_fd,
            events.as_mut_ptr(),
            events.len() as i32,
            timeout
        ))?;

        for event in &events[0..res as usize] {
            if let Some(Some(waker)) = self.interest_set.get(&(event.u64 as RawFd)) {
                waker.wake_by_ref();
            }
        }

        Ok(res)
    }
}
