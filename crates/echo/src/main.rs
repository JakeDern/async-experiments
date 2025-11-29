use echo::executor::Executor;
use echo::tcp::TcpListener;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::os::fd::AsRawFd;

thread_local! {
    pub static EXECUTOR: RefCell<Option<Executor>> = RefCell::new(None);
}

// Each thread gets its own copy, initialized to None

fn main() {
    EXECUTOR.with(|exec| {
        let executor = Executor::new();
        *exec.borrow_mut() = Some(executor);
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr, 128).unwrap();
    echo::sys::close_socket(listener.as_raw_fd()).unwrap();
}
