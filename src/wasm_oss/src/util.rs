use crate::lwip_error::LwipError;
use std::net::Ipv4Addr;

pub fn ip_addr_to_u32(addr: &str) -> Result<u32, LwipError> {
    let addr: Ipv4Addr = addr.parse().map_err(|_| LwipError::IllegalArgument)?;
    return Ok(u32::from_be_bytes(addr.octets()).to_be());
}
