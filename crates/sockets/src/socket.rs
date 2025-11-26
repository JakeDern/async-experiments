#[derive(Debug)]
pub struct Socket<T> {
    num: i32,
    state: std::marker::PhantomData<T>,
}

#[derive(Debug)]
pub struct Unbound;
#[derive(Debug)]
pub struct Bound;

impl Socket<Unbound> {
    fn new(addr: &libc::addrinfo) -> anyhow::Result<Self> {
        let socket = unsafe { libc::socket(addr.ai_family, addr.ai_socktype, addr.ai_protocol) };
        if socket < 0 {
            return Err(anyhow::anyhow!("Failed to create socket"));
        }

        Ok(Socket {
            num: socket,
            state: std::marker::PhantomData,
        })
    }

    pub fn bind(self, addr: &libc::addrinfo) -> anyhow::Result<Socket<Bound>> {
        let bind_res = unsafe { libc::bind(self.num, addr.ai_addr, addr.ai_addrlen) };
        if bind_res < 0 {
            return Err(anyhow::anyhow!("Failed to bind socket"));
        }

        return Ok(Socket::<Bound> {
            num: self.num,
            state: std::marker::PhantomData,
        });
    }
}

impl TryFrom<&libc::addrinfo> for Socket<Unbound> {
    type Error = anyhow::Error;

    fn try_from(value: &libc::addrinfo) -> Result<Self, Self::Error> {
        Socket::<Unbound>::new(value)
    }
}
