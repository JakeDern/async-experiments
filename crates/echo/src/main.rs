use echo::runtime;
use echo::tcp::TcpListener;
use std::net::SocketAddr;

// Each thread gets its own copy, initialized to None
fn main() {
    runtime::run(async {
        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = TcpListener::bind(addr, 128).unwrap();
        while let Ok((fd, addr)) = listener.accept().await {
            println!("Accepted connection from {}", addr);
            unsafe {
                libc::close(fd);
            }
        }
    })
    .unwrap();
}
