fn main() -> anyhow::Result<()> {
    let addr_info = sockets::get_local_addr_info("8080").unwrap();
    let socket = sockets::socket::SocketFd::try_from(&addr_info).unwrap();
    let socket = socket.connect(&addr_info)?;
    println!("Connected to socket {:?}", socket);
    let nbytes = socket.send("Hello, World".as_bytes())?;
    println!("Sent {} bytes", nbytes);
    Ok(())
}
