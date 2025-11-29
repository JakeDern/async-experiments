use std::io;

use crate::executor::{self, Executor};
use crate::{EXECUTOR, REACTOR, reactor};

pub fn block_on<F, T>(fut: F) -> io::Result<T>
where
    F: Future<Output = T> + 'static,
    T: 'static,
{
    REACTOR.with(|reactor| {
        if reactor.borrow().is_some() {
            panic!("Reactor already started on this thread");
        }
        // TODO: Error handling
        let react = reactor::Reactor::new().unwrap();
        *reactor.borrow_mut() = Some(react);
    });

    EXECUTOR.with(|exec| {
        if exec.borrow().is_some() {
            panic!("Runtime already started on this thread");
        }
        let executor = Executor::new();
        *exec.borrow_mut() = Some(executor);
    });

    let _ = executor::spawn(fut);
    loop {
        executor::make_progress();
        let mut events = [libc::epoll_event { events: 0, u64: 0 }; 128];
        reactor::wait_and_wake(&mut events, -1)?;
    }
}
