use sockets::sockaddr_to_string;

fn main() -> anyhow::Result<()> {
    let addr_info = sockets::get_local_addr_info("8080").unwrap();
    let socket = sockets::socket::SocketFd::try_from(&addr_info).unwrap();
    let socket = socket.bind(&addr_info)?;
    let socket = socket.listen(10)?;
    println!("Created and bound socket: {:?}", socket);
    let (fd, client_addr) = socket.accept().unwrap();
    println!(
        "Accepted socket. fd: {:?}, addr: {:?}",
        fd,
        sockaddr_to_string(&client_addr).unwrap()
    );

    let buf = b"Hello, world!\n";
    fd.send(buf)?;

    Ok(())
}
