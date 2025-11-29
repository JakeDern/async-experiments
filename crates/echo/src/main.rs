use echo::tcp::TcpListener;
use std::net::SocketAddr;
use std::os::fd::AsRawFd;

fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr, 128).unwrap();
    echo::sys::close_socket(listener.as_raw_fd()).unwrap();
}
