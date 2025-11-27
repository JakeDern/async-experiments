use std::mem;
use std::net::{Ipv4Addr, Ipv6Addr};

use libc::{AF_INET, AI_PASSIVE, SOCK_STREAM};

pub mod socket;

pub fn get_local_addr_info(port: &str) -> anyhow::Result<libc::addrinfo> {
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_flags = AI_PASSIVE;

    let mut results: *mut libc::addrinfo = std::ptr::null_mut();
    unsafe {
        let info = libc::getaddrinfo(
            std::ptr::null(),
            std::ffi::CString::new(port).unwrap().as_ptr(),
            &hints,
            &mut results,
        );

        if info != 0 {
            return Err(anyhow::anyhow!(
                "getaddrinfo failed with error code: {}",
                info
            ));
        };

        if results.is_null() {
            return Err(anyhow::anyhow!("getaddrinfo returned no results"));
        }
    }

    Ok(unsafe { *results })
}

pub fn sockaddr_to_string(addr: &libc::sockaddr) -> Option<String> {
    match addr.sa_family as i32 {
        libc::AF_INET => {
            let addr_in = unsafe { &*(addr as *const _ as *const libc::sockaddr_in) };
            let ip = Ipv4Addr::from(u32::from_be(addr_in.sin_addr.s_addr));
            Some(ip.to_string())
        }
        libc::AF_INET6 => {
            let addr_in6 = unsafe { &*(addr as *const _ as *const libc::sockaddr_in6) };
            let ip = Ipv6Addr::from(addr_in6.sin6_addr.s6_addr);
            Some(ip.to_string())
        }
        _ => None,
    }
}
