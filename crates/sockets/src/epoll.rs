use libc::pollfd;

use crate::socket;

pub struct Set {
    pub fds: Vec<pollfd>,
}

impl Set {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            fds: Vec::with_capacity(cap),
        }
    }

    pub fn add(&mut self, fd: socket::RawFd, events: i16) {
        self.fds.push(pollfd {
            fd: fd.0,
            events,
            revents: 0,
        });
    }
}
