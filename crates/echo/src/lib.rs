use std::cell::RefCell;

use self::executor::Executor;

pub mod channel;
pub mod echo;
pub mod executor;
pub mod io;
pub mod reactor;
pub mod runtime;
pub mod sys;
pub mod tcp;

thread_local! {
    pub(crate) static EXECUTOR: RefCell<Option<Executor>> = RefCell::new(None);
    pub(crate) static REACTOR: RefCell<Option<reactor::Reactor>> = RefCell::new(None);
}
