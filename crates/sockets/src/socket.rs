use crate::RawFd;

#[derive(Debug)]
pub struct SocketFd<T> {
    fd: RawFd,
    state: std::marker::PhantomData<T>,
}

impl<T> SocketFd<T> {
    pub fn into_inner(self) -> RawFd {
        self.fd
    }

    pub fn as_ref(&self) -> &RawFd {
        &self.fd
    }

    pub fn close(self) -> anyhow::Result<()> {
        let res = unsafe { libc::close(self.fd.0) };

        if res < 0 {
            return Err(anyhow::anyhow!("Failed to close socket: {}", res));
        }

        Ok(())
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

impl SocketFd<Unbound> {
    // TODO: What we should do here is keep trying to bind to each address in
    // the addrinfo list until one works.
    pub fn new(addr: &libc::addrinfo) -> anyhow::Result<Self> {
        let socket = unsafe { libc::socket(addr.ai_family, addr.ai_socktype, addr.ai_protocol) };
        if socket < 0 {
            return Err(anyhow::anyhow!("Failed to create socket"));
        }

        Ok(SocketFd {
            fd: RawFd(socket),
            state: std::marker::PhantomData,
        })
    }

    pub fn bind(self, addr: &libc::addrinfo) -> anyhow::Result<SocketFd<Bound>> {
        let bind_res = unsafe { libc::bind(self.fd.0, addr.ai_addr, addr.ai_addrlen) };
        if bind_res < 0 {
            return Err(anyhow::anyhow!("Failed to bind socket: {}", bind_res));
        }

        Ok(SocketFd::<Bound> {
            fd: self.into_inner(),
            state: std::marker::PhantomData,
        })
    }

    pub fn connect(self, addr: &libc::addrinfo) -> anyhow::Result<SocketFd<Connected>> {
        connect_socket(self, addr)
    }
}

impl SocketFd<Bound> {
    pub fn listen(self, backlog: usize) -> anyhow::Result<SocketFd<Listening>> {
        let res = unsafe { libc::listen(self.fd.0, backlog as i32) };
        if res < 0 {
            return Err(anyhow::anyhow!("Failed to listen on socket: {}", res));
        }

        Ok(SocketFd::<Listening> {
            fd: self.into_inner(),
            state: std::marker::PhantomData,
        })
    }

    // We can receive a datagram on a bound socket even if we're not connected
    pub fn recv(&self, buf: &[u8]) -> anyhow::Result<usize> {
        self.fd.recv(buf)
    }

    pub fn connect(self, addr: &libc::addrinfo) -> anyhow::Result<SocketFd<Connected>> {
        connect_socket(self, addr)
    }
}

impl TryFrom<&libc::addrinfo> for SocketFd<Unbound> {
    type Error = anyhow::Error;

    fn try_from(value: &libc::addrinfo) -> Result<Self, Self::Error> {
        SocketFd::<Unbound>::new(value)
    }
}

impl SocketFd<Listening> {
    // TODO: Consider that sockaddr might only be large enough for IPV4 and
    // we may need to use sockaddr_storage to allow for IPV6
    //
    // Accept takes sockaddr though, probably that works because the layout is
    // the same and we can just cast it.
    pub fn accept(&self) -> anyhow::Result<(SocketFd<Connected>, libc::sockaddr)> {
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

        Ok((
            SocketFd::<Connected> {
                fd: RawFd(res),
                state: std::marker::PhantomData,
            },
            client_addr,
        ))
    }
}

impl SocketFd<Connected> {
    pub fn send(&self, buf: &[u8]) -> anyhow::Result<usize> {
        self.fd.send(buf)
    }

    pub fn recv(&self, buf: &[u8]) -> anyhow::Result<usize> {
        self.fd.recv(buf)
    }
}

fn connect_socket<T>(
    socket: SocketFd<T>,
    addr: &libc::addrinfo,
) -> anyhow::Result<SocketFd<Connected>> {
    let res = unsafe { libc::connect(socket.fd.0, addr.ai_addr, addr.ai_addrlen) };
    if res < 0 {
        return Err(anyhow::anyhow!("Failed to connect socket: {}", res));
    }

    Ok(SocketFd::<Connected> {
        fd: socket.into_inner(),
        state: std::marker::PhantomData,
    })
}
