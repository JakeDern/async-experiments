use libc::{EPOLL_CTL_ADD, EPOLLERR, EPOLLIN, EPOLLOUT};

use crate::RawFd;

pub struct EpollFd(RawFd);

impl EpollFd {
    pub fn new() -> anyhow::Result<EpollFd> {
        let res = unsafe { libc::epoll_create1(0) };
        if res < 0 {
            return Err(anyhow::anyhow!("epoll_create failed: {}", res));
        }

        Ok(EpollFd(RawFd(res)))
    }

    fn as_raw(&self) -> i32 {
        self.0.0
    }

    pub fn add(&self, fd: &RawFd) -> anyhow::Result<()> {
        let mut event = libc::epoll_event {
            events: (EPOLLIN | EPOLLOUT | EPOLLERR) as u32,
            u64: fd.0 as u64,
        };

        let res = unsafe { libc::epoll_ctl(self.as_raw(), EPOLL_CTL_ADD, fd.0, &mut event) };
        if res < 0 {
            return Err(anyhow::anyhow!("epoll_ctl failed: {}", res));
        }

        Ok(())
    }

    pub fn remove(&self, fd: &RawFd) -> anyhow::Result<()> {
        let res = unsafe {
            libc::epoll_ctl(
                self.as_raw(),
                libc::EPOLL_CTL_DEL,
                fd.0,
                std::ptr::null_mut(),
            )
        };
        if res < 0 {
            return Err(anyhow::anyhow!("epoll_ctl failed: {}", res));
        }

        Ok(())
    }

    pub fn wait(&self, events: &mut [libc::epoll_event], timeout: i32) -> anyhow::Result<usize> {
        let res = unsafe {
            libc::epoll_wait(
                self.as_raw(),
                events.as_mut_ptr(),
                events.len() as i32,
                timeout,
            )
        };
        if res < 0 {
            return Err(anyhow::anyhow!("epoll_wait failed: {}", res));
        }

        Ok(res as usize)
    }
}
