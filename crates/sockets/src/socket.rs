#[derive(Debug)]
pub struct RawFd(i32);

impl RawFd {
    pub fn send(&self, buf: &[u8]) -> anyhow::Result<usize> {
        // TODO: What kind of flags for the last argument might we care about?
        let res = unsafe { libc::send(self.0, buf.as_ptr() as *const _, buf.len(), 0) };
        if res < 0 {
            return Err(anyhow::anyhow!("Failed to send data: {}", res));
        }
        Ok(res as usize)
    }

    pub fn recv(&self, buf: &[u8]) -> anyhow::Result<usize> {
        let res = unsafe { libc::recv(self.0, buf.as_ptr() as *mut _, buf.len(), 0) };
        if res < 0 {
            return Err(anyhow::anyhow!("Failed to receive data: {}", res));
        }

        Ok(res as usize)
    }
}

impl Drop for RawFd {
    fn drop(&mut self) {
        unsafe {
            let res = libc::close(self.0);
            println!("File Descriptor '{}', closed with '{}'", self.0, res);
        }
    }
}

#[derive(Debug)]
pub struct Socket<T> {
    fd: RawFd,
    state: std::marker::PhantomData<T>,
}

impl<T> Socket<T> {
    pub fn into_inner(self) -> RawFd {
        self.fd
    }
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
    // TODO: What we should do here is keep trying to bind to each address in
    // the addrinfo list until one works.
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
            return Err(anyhow::anyhow!("Failed to bind socket: {}", bind_res));
        }

        Ok(Socket::<Bound> {
            fd: self.into_inner(),
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
            return Err(anyhow::anyhow!("Failed to listen on socket: {}", res));
        }

        Ok(Socket::<Listening> {
            fd: self.into_inner(),
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

impl Socket<Listening> {
    // TODO: Consider that sockaddr might only be large enough for IPV4 and
    // we may need to use sockaddr_storage to allow for IPV6
    //
    // Accept takes sockaddr though, probably that works because the layout is
    // the same and we can just cast it.
    pub fn accept(&self) -> anyhow::Result<(RawFd, libc::sockaddr)> {
        let mut client_addr: libc::sockaddr = unsafe { std::mem::zeroed() };
        let mut addr_len = std::mem::size_of::<libc::sockaddr>() as libc::socklen_t;
        let res = unsafe {
            libc::accept(
                self.fd.0,
                &mut client_addr as *mut _,
                &mut addr_len as *mut _,
            )
        };

        if res < 0 {
            return Err(anyhow::anyhow!("Failed to accept connection: {}", res));
        }

        Ok((RawFd(res), client_addr))
    }
}

impl Socket<Connected> {
    pub fn recv(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
        self.fd.recv(buf)
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

    Ok(Socket::<Connected> {
        fd: socket.into_inner(),
        state: std::marker::PhantomData,
    })
}
