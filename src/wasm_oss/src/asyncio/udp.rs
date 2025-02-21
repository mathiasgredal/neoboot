use super::socket::Socket;
use crate::ffi;
use crate::lwip_error::LwipError;
use crate::util::ip_addr_to_u32;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct UdpSocket {
    pub socket: Socket,
}

impl UdpSocket {
    pub fn create() -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_net_socket_new_udp() };
        if socket < 0 {
            return Err(LwipError::from_code(socket));
        }
        Ok(Self {
            socket: Socket {
                socket: Rc::new(RefCell::new(socket)),
            },
        })
    }

    pub fn bind(&self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result = unsafe {
            ffi::env_net_socket_bind(self.socket.socket.borrow().clone(), addr, port.into())
        };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }
        Ok(())
    }

    pub fn connect(&self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result = unsafe {
            ffi::env_net_socket_connect(self.socket.socket.borrow().clone(), addr, port.into())
        };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }
        Ok(())
    }
}
