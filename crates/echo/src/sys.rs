use std::io;
use std::net::SocketAddr;
use std::os::fd::RawFd;

use libc::c_int;

#[macro_export]
macro_rules! syscall {
    ($fn_name:ident ( $($arg:expr),* $(,)? ) ) => {
        {
            let res = unsafe { libc::$fn_name( $($arg),* ) };
            if res < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(res)
            }
        }
    };
}

#[repr(i32)]
pub enum EpollEventKind {
    Readable = libc::EPOLLIN,
    Writable = libc::EPOLLOUT,
}

pub fn epoll_create() -> io::Result<RawFd> {
    syscall!(epoll_create1(0))
}

pub fn sock_listen(fd: RawFd, backlog: c_int) -> io::Result<i32> {
    syscall!(listen(fd, backlog))
}

pub fn sock_accept_nonblock(fd: RawFd) -> io::Result<(RawFd, SocketAddr)> {
    let mut client_addr: libc::sockaddr = unsafe { std::mem::zeroed() };
    let mut addr_len = std::mem::size_of::<libc::sockaddr>() as libc::socklen_t;
    let res = syscall!(accept4(
        fd,
        &mut client_addr as *mut _,
        &mut addr_len as *mut _,
        libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
    ))?;

    return Ok((res, sockaddr_to_socketaddr(client_addr)));
}

pub fn sock_bind(fd: RawFd, addr: SocketAddr, reuseport: bool) -> io::Result<i32> {
    let addrinfo = socketaddr_to_addrinfo(addr);
    if reuseport {
        set_so_reuseport(fd)?;
    }
    syscall!(bind(fd, addrinfo.ai_addr, addrinfo.ai_addrlen))
}

pub fn open_tcp_socket(addr: SocketAddr) -> io::Result<RawFd> {
    match addr {
        SocketAddr::V4(_) => open_socket(libc::AF_INET, libc::SOCK_STREAM),
        SocketAddr::V6(_) => open_socket(libc::AF_INET6, libc::SOCK_STREAM),
    }
}

/// Open a non-blocking socket with close-on-exec
pub fn open_socket(domain: c_int, typ: c_int) -> io::Result<RawFd> {
    let typ = typ | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC;
    syscall!(socket(domain, typ, 0))
}

pub fn set_so_reuseport(fd: RawFd) -> io::Result<i32> {
    let optval: c_int = 1;
    syscall!(setsockopt(
        fd,
        libc::SOL_SOCKET,
        libc::SO_REUSEPORT,
        &optval as *const c_int as *const _,
        std::mem::size_of::<c_int>() as u32,
    ))
}

pub fn sockaddr_to_socketaddr(addr: libc::sockaddr) -> SocketAddr {
    match addr.sa_family as c_int {
        libc::AF_INET => {
            let sockaddr_in: libc::sockaddr_in = unsafe { std::mem::transmute(addr) };
            let ip =
                std::net::Ipv4Addr::from(u32::from_be(sockaddr_in.sin_addr.s_addr).to_ne_bytes());
            let port = u16::from_be(sockaddr_in.sin_port);
            SocketAddr::V4(std::net::SocketAddrV4::new(ip, port))
        }
        libc::AF_INET6 => {
            todo!()
        }
        _ => panic!("Unsupported address family"),
    }
}

pub fn socketaddr_to_addrinfo(addr: SocketAddr) -> libc::addrinfo {
    match addr {
        SocketAddr::V4(v4_addr) => {
            let sockaddr_in = libc::sockaddr_in {
                sin_family: libc::AF_INET as u16,
                sin_port: v4_addr.port().to_be(),
                sin_addr: libc::in_addr {
                    s_addr: u32::from_ne_bytes(v4_addr.ip().octets()).to_be(),
                },
                sin_zero: [0; 8],
            };
            let mut addrinfo: libc::addrinfo = unsafe { std::mem::zeroed() };
            addrinfo.ai_family = libc::AF_INET;
            addrinfo.ai_socktype = libc::SOCK_STREAM;
            addrinfo.ai_addrlen = std::mem::size_of::<libc::sockaddr_in>() as u32;
            addrinfo.ai_addr = Box::into_raw(Box::new(sockaddr_in)) as *mut libc::sockaddr;

            addrinfo
        }
        SocketAddr::V6(v6_addr) => {
            let sockaddr_in6 = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as u16,
                sin6_port: v6_addr.port().to_be(),
                sin6_flowinfo: v6_addr.flowinfo(),
                sin6_addr: libc::in6_addr {
                    s6_addr: v6_addr.ip().octets(),
                },
                sin6_scope_id: v6_addr.scope_id(),
            };
            let mut addrinfo: libc::addrinfo = unsafe { std::mem::zeroed() };
            addrinfo.ai_family = libc::AF_INET6;
            addrinfo.ai_socktype = libc::SOCK_STREAM;
            addrinfo.ai_addrlen = std::mem::size_of::<libc::sockaddr_in6>() as u32;
            addrinfo.ai_addr = Box::into_raw(Box::new(sockaddr_in6)) as *mut libc::sockaddr;

            addrinfo
        }
    }
}

pub fn close_socket(fd: RawFd) -> io::Result<i32> {
    syscall!(close(fd))
}
