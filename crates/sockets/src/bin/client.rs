fn main() -> anyhow::Result<()> {
    let addr_info = sockets::get_local_addr_info("8080").unwrap();
    let socket = sockets::socket::SocketFd::try_from(&addr_info).unwrap();
    let socket = socket.connect(&addr_info)?;
    println!("Connected to socket {:?}", socket);
    let mut buf = [0; 1024];
    let nbytes = socket.recv(&mut buf)?;
    println!(
        "Received {} bytes: {:?}",
        nbytes,
        String::from_utf8_lossy(&buf[..nbytes])
    );
    Ok(())
}
