fn main() -> anyhow::Result<()> {
    let addr_info = sockets::get_local_addr_info("8080").unwrap();
    let socket = sockets::socket::Socket::try_from(&addr_info).unwrap();
    let socket = socket.connect(&addr_info)?;
    println!("Connected to socket {:?}", socket);
    Ok(())
}
