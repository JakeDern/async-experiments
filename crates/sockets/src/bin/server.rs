use libc::epoll_event;
use sockets::{RawFd, epoll, sockaddr_to_string};

fn main() -> anyhow::Result<()> {
    let addr_info = sockets::get_local_addr_info("8080").unwrap();
    let listen_sock = sockets::socket::SocketFd::try_from(&addr_info).unwrap();
    let listen_sock = listen_sock.bind(&addr_info)?;
    let listen_sock = listen_sock.listen(10)?;
    println!("Listening on port 8080");

    let epoll_fd = epoll::EpollFd::new().unwrap();
    epoll_fd.add(listen_sock.as_ref()).unwrap();

    let mut events = [epoll_event { events: 0, u64: 0 }; 32];
    loop {
        let len = epoll_fd.wait(&mut events, -1).unwrap();
        let read_events = &events[..len];
        for event in read_events {
            if event.u64 == listen_sock.as_ref().inner() as u64 {
                let (client_sock, client_addr) = listen_sock.accept()?;
                println!(
                    "Accepted connection from {}",
                    sockaddr_to_string(&client_addr).unwrap()
                );
                epoll_fd.add(client_sock.as_ref())?;
            } else {
                if (event.events & (libc::EPOLLIN as u32)) != 0 {
                    let client_fd = RawFd::new(event.u64 as i32);

                    let mut buffer = [0u8; 1024];
                    let nbytes = client_fd.recv(&mut buffer)?;
                    if nbytes == 0 {
                        println!("Nothing to read");
                        continue;
                    } else {
                        println!(
                            "Received data: {}",
                            String::from_utf8_lossy(&buffer[..nbytes])
                        );
                    }
                }
            }
        }
    }
}
