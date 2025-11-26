#[derive(Debug)]
pub struct RawFd(i32);

impl Drop for RawFd {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe {
                libc::close(self.0);
            }
        }
    }
}

#[derive(Debug)]
pub struct Socket<T> {
    fd: RawFd,
    state: std::marker::PhantomData<T>,
}

#[derive(Debug)]
pub struct Unbound;
#[derive(Debug)]
pub struct Bound;
#[derive(Debug)]
pub struct Connected;
#[derive(Debug)]
pub struct Listening;

impl Socket<Unbound> {
    pub fn new(addr: &libc::addrinfo) -> anyhow::Result<Self> {
        let socket = unsafe { libc::socket(addr.ai_family, addr.ai_socktype, addr.ai_protocol) };
        if socket < 0 {
            return Err(anyhow::anyhow!("Failed to create socket"));
        }

        Ok(Socket {
            fd: RawFd(socket),
            state: std::marker::PhantomData,
        })
    }

    pub fn bind(self, addr: &libc::addrinfo) -> anyhow::Result<Socket<Bound>> {
        let bind_res = unsafe { libc::bind(self.fd.0, addr.ai_addr, addr.ai_addrlen) };
        if bind_res < 0 {
            return Err(anyhow::anyhow!("Failed to bind socket"));
        }

        let Self { fd, .. } = self;
        Ok(Socket::<Bound> {
            fd,
            state: std::marker::PhantomData,
        })
    }

    pub fn connect(self, addr: &libc::addrinfo) -> anyhow::Result<Socket<Connected>> {
        connect_socket(self, addr)
    }
}

impl Socket<Bound> {
    pub fn listen(self, backlog: usize) -> anyhow::Result<Socket<Listening>> {
        let res = unsafe { libc::listen(self.fd.0, backlog as i32) };
        if res < 0 {
            return Err(anyhow::anyhow!("Failed to listen on socket"));
        }

        let Self { fd, .. } = self;
        Ok(Socket::<Listening> {
            fd,
            state: std::marker::PhantomData,
        })
    }

    pub fn connect(self, addr: &libc::addrinfo) -> anyhow::Result<Socket<Connected>> {
        connect_socket(self, addr)
    }
}

impl TryFrom<&libc::addrinfo> for Socket<Unbound> {
    type Error = anyhow::Error;

    fn try_from(value: &libc::addrinfo) -> Result<Self, Self::Error> {
        Socket::<Unbound>::new(value)
    }
}

fn connect_socket<T>(
    socket: Socket<T>,
    addr: &libc::addrinfo,
) -> anyhow::Result<Socket<Connected>> {
    let res = unsafe { libc::connect(socket.fd.0, addr.ai_addr, addr.ai_addrlen) };
    if res < 0 {
        return Err(anyhow::anyhow!("Failed to connect socket: {}", res));
    }

    let Socket::<T> { fd, .. } = socket;

    Ok(Socket::<Connected> {
        fd,
        state: std::marker::PhantomData,
    })
}
