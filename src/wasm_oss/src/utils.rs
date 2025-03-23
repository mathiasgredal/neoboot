use crate::errors::lwip_error::LwipError;
use crate::ffi;
use std::net::Ipv4Addr;

pub mod logging;
pub mod msgpack;
pub mod panic;

// Utility functions
pub fn ip_addr_to_u32(addr: &str) -> Result<u32, LwipError> {
    let addr: Ipv4Addr = addr.parse().map_err(|_| LwipError::IllegalArgument)?;
    Ok(u32::from_be_bytes(addr.octets()).to_be())
}

pub fn sys_print(s: &str) {
    unsafe {
        ffi::env_print(s.as_ptr(), s.len() as u32);
    }
}

pub fn sys_get_env(key: &str) -> Result<String, LwipError> {
    // Allocate a 512 byte buffer for the return value
    let mut value = vec![0; 512];

    let result = unsafe {
        ffi::env_get_env(
            key.as_ptr(),
            key.len() as u32,
            value.as_ptr(),
            value.len() as u32,
        )
    };

    // If the result is 0 or negative, return an error
    if result < 0 {
        return Err(LwipError::IllegalArgument);
    }

    // Convert the buffer to a string, use result as the length
    let value = String::from_utf8(value[..result as usize].to_vec()).unwrap();
    Ok(value)
}

pub fn parse_int(s: &str) -> std::result::Result<u64, std::num::ParseIntError> {
    if let Some(s) = s.strip_prefix("0x") {
        u64::from_str_radix(s, 16)
    } else if let Some(s) = s.strip_prefix("0o") {
        u64::from_str_radix(s, 8)
    } else if let Some(s) = s.strip_prefix("0b") {
        u64::from_str_radix(s, 2)
    } else {
        s.parse::<u64>()
    }
}
