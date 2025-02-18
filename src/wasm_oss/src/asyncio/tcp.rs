use log::{error, info};
use std::cell::RefCell;
use std::future::Future;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::ffi;
use crate::lwip_error::LwipError;

fn ip_addr_to_u32(addr: &str) -> Result<u32, LwipError> {
    let addr: Ipv4Addr = addr.parse().map_err(|_| LwipError::IllegalArgument)?;
    return Ok(u32::from_be_bytes(addr.octets()).to_be());
}

pub struct Socket {
    pub socket: i32,
}

impl Socket {
    pub async fn read(&self, len: u16) -> Result<Vec<u8>, LwipError> {
        struct Read {
            socket: i32,
            buf: Vec<u8>,
            len: u16,
        }

        impl Future for Read {
            type Output = Result<Vec<u8>, LwipError>;

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };

                let result = unsafe {
                    ffi::env_net_socket_read(self.socket, self.buf.as_ptr(), self.len as u32)
                };

                if result == LwipError::WouldBlock.to_code() {
                    return Poll::Pending;
                }

                if result >= 0 {
                    let ret_buf = self.buf.iter().take(result as usize).cloned().collect();
                    return Poll::Ready(Ok(ret_buf));
                }

                return Poll::Ready(Err(LwipError::from_code(result)));
            }
        }

        // 1. Heap malloc len bytes
        let buf: Vec<u8> = vec![0x0; len as usize];

        // 2. Call env_socket_read
        let result = Read {
            socket: self.socket,
            buf,
            len,
        }
        .await;

        if result.is_err() {
            return Err(result.err().unwrap());
        }

        // 3. If no error, return the read buffer
        return Ok(result.unwrap());
    }

    pub async fn write(&self, buf: &[u8]) -> Result<usize, LwipError> {
        struct Write {
            socket: i32,
        }

        impl Future for Write {
            type Output = Result<(), LwipError>;

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };

                let err = unsafe { ffi::env_net_socket_write_poll(self.socket) };

                if err == LwipError::WouldBlock.to_code() {
                    return Poll::Pending;
                }

                if err == LwipError::Ok.to_code() {
                    return Poll::Ready(Ok(()));
                }

                return Poll::Ready(Err(LwipError::from_code(err)));
            }
        }

        // 1. Call env_socket_write
        let result =
            unsafe { ffi::env_net_socket_write(self.socket, buf.as_ptr(), buf.len() as u32) };

        if result != LwipError::Ok.to_code() {
            let err = LwipError::from_code(result);
            info!("Error initial writing: {:?}", err);
            return Err(err);
        }

        // 3. If no error, return a tcp write future
        let result = Write {
            socket: self.socket,
        }
        .await;

        if result.is_err() {
            return Err(result.err().unwrap());
        }

        // 4. If no error, return the number of bytes written
        return Ok(buf.len());
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        info!("Closing socket: {}", self.socket);
        let result = unsafe { ffi::env_net_socket_free(self.socket) };
        if result != LwipError::Ok.to_code() {
            error!("Failed to close socket: {}", LwipError::from_code(result));
        }
    }
}

pub struct TcpSocket {
    pub socket: Socket,
}

impl TcpSocket {
    pub fn create() -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_net_socket_new_tcp() };
        if socket < 0 {
            return Err(LwipError::from_code(socket));
        }
        Ok(Self {
            socket: Socket { socket },
        })
    }

    pub async fn connect_raw(&mut self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        struct TcpConnection {
            socket: i32,
        }

        impl Future for TcpConnection {
            type Output = Result<(), LwipError>;

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };

                let err = unsafe { ffi::env_net_socket_connect_poll(self.socket) };

                if err == LwipError::WouldBlock.to_code() {
                    return Poll::Pending;
                }

                if err == LwipError::Ok.to_code() {
                    return Poll::Ready(Ok(()));
                }

                return Poll::Ready(Err(LwipError::from_code(err)));
            }
        }

        let addr = ip_addr_to_u32(addr_str)?;
        let result: i32 =
            unsafe { ffi::env_net_socket_connect(self.socket.socket, addr, port.into()) };

        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        // Wait for connection
        let result = TcpConnection {
            socket: self.socket.socket,
        }
        .await;

        if result.is_err() {
            return Err(result.err().unwrap());
        }

        Ok(())
    }

    pub fn bind(&self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result = unsafe { ffi::env_net_socket_bind(self.socket.socket, addr, port.into()) };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }
        Ok(())
    }

    pub fn listen(&self, backlog: u32) -> Result<(), LwipError> {
        let result = unsafe { ffi::env_net_socket_listen(self.socket.socket, backlog) };

        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        Ok(())
    }

    pub async fn accept(&self) -> Result<Self, LwipError> {
        struct TcpAccept {
            socket: i32,
        }

        impl Future for TcpAccept {
            type Output = Result<TcpSocket, LwipError>;

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };
                let result = unsafe { ffi::env_net_socket_accept_poll(self.socket) };

                if result == LwipError::WouldBlock.to_code() {
                    return Poll::Pending;
                }

                if result < 0 {
                    return Poll::Ready(Err(LwipError::from_code(result)));
                }

                return Poll::Ready(Ok(TcpSocket {
                    socket: Socket { socket: result },
                }));
            }
        }

        let result = unsafe { ffi::env_net_socket_accept(self.socket.socket) };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        let accept_result = TcpAccept {
            socket: self.socket.socket,
        }
        .await;

        if accept_result.is_err() {
            return Err(accept_result.err().unwrap());
        }

        Ok(accept_result.unwrap())
    }
}

#[derive(Clone)]
pub struct UdpSocket {
    pub socket: Rc<RefCell<Socket>>,
}

impl UdpSocket {
    pub fn create() -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_net_socket_new_udp() };
        if socket < 0 {
            return Err(LwipError::from_code(socket));
        }
        Ok(Self {
            socket: Rc::new(RefCell::new(Socket { socket })),
        })
    }

    pub fn bind(&self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result =
            unsafe { ffi::env_net_socket_bind(self.socket.borrow().socket, addr, port.into()) };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }
        Ok(())
    }

    pub fn connect(&self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result =
            unsafe { ffi::env_net_socket_connect(self.socket.borrow().socket, addr, port.into()) };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }
        Ok(())
    }
}
