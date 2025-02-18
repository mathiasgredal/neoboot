use log::error;
use std::future::Future;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::task::{Context, Poll};

use log::info;

use crate::ffi;
use crate::lwip_error::LwipError;

fn ip_addr_to_u32(addr: &str) -> Result<u32, LwipError> {
    let addr: Ipv4Addr = addr.parse().map_err(|_| LwipError::IllegalArgument)?;
    return Ok(u32::from_be_bytes(addr.octets()).to_be());
}

struct TcpConnection {
    socket: u64,
}

impl Future for TcpConnection {
    type Output = Result<(), LwipError>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { ffi::env_lwip_rx() };

        let err = unsafe { ffi::env_socket_check_connection(self.socket) };

        if err == LwipError::InProgress.to_code() {
            return Poll::Pending;
        }

        if err == LwipError::Ok.to_code() {
            return Poll::Ready(Ok(()));
        }

        return Poll::Ready(Err(LwipError::from_code(err)));
    }
}

struct TcpWrite {
    socket: u64,
}

impl Future for TcpWrite {
    type Output = Result<(), LwipError>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { ffi::env_lwip_rx() };

        let err = unsafe { ffi::env_socket_all_writes_acked(self.socket) };

        if err == LwipError::WouldBlock.to_code() {
            return Poll::Pending;
        }

        if err == LwipError::Ok.to_code() {
            return Poll::Ready(Ok(()));
        }

        return Poll::Ready(Err(LwipError::from_code(err)));
    }
}

struct TcpRead {
    socket: u64,
    buf: Vec<u8>,
    len: u16,
}

impl Future for TcpRead {
    type Output = Result<Vec<u8>, LwipError>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { ffi::env_lwip_rx() };

        let result = unsafe {
            ffi::env_socket_read(
                self.socket,
                self.buf.as_ptr(),
                self.len as u32,
            )
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


struct TcpAccept {
    socket: u64,
}

impl Future for TcpAccept {
    type Output = Result<TcpSocket, LwipError>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { ffi::env_lwip_rx() };

        // TODO: Make some other call before, to check if the connection is still valid

        let result = unsafe { ffi::env_socket_accept_claim_connection(self.socket) };

        if result == 0 {
            return Poll::Pending;
        }

        return Poll::Ready(Ok(TcpSocket { socket: result }));
    }
}

#[derive(Clone)]
pub struct TcpSocket {
    pub socket: u64,
}

impl TcpSocket {
    pub async fn create() -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_socket_create() };
        if socket == 0 {
            return Err(LwipError::ConnectionAborted);
        }
        Ok(Self { socket })
    }

    pub async fn connect(
        &mut self,
        addr_str: &str,
        port: u16,
    ) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result: i32 = unsafe { ffi::env_socket_connect(self.socket, addr, port.into()) };

        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        // Wait for connection
        let result = TcpConnection { socket: self.socket }.await;

        // If the connection attempt failed, close the socket
        if result.is_err() {
            info!("Failed to connect to {}:{}", addr_str, port);
            let close_result = unsafe { ffi::env_socket_close(self.socket) };
            if close_result != LwipError::Ok.to_code() {
                error!(
                    "Failed to close socket: {}",
                    LwipError::from_code(close_result)
                );
                error!("NOT IMPLEMENTED: socket closing retry functionality, leaking socket");
            }
            return Err(result.err().unwrap());
        }

        Ok(())
    }

    pub async fn bind(&mut self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result = unsafe { ffi::env_socket_bind(self.socket, addr, port.into()) };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }
        Ok(())
    }

    pub async fn listen(&mut self, backlog: u32) -> Result<(), LwipError> {
        let result = unsafe { ffi::env_socket_listen(self.socket, backlog) };

        info!("Listening on socket: {}", self.socket);
        info!("Result: {}", result);
        if result == 0 {
            return Err(LwipError::InvalidValue);
        }

        self.socket = result;
        Ok(())
    }

    pub async fn accept(&self) -> Result<Self, LwipError> {
        let result = unsafe { ffi::env_socket_accept(self.socket) };
        info!("Accepting on socket: {}", self.socket);
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        let accept_result = TcpAccept { socket: self.socket }.await;

        if accept_result.is_err() {
            info!("Failed to accept connection");
            return Err(accept_result.err().unwrap());
        }

        Ok(accept_result.unwrap())
    }

    pub async fn read(&self, len: u16) -> Result<Vec<u8>, LwipError> {
        // 1. Heap malloc len bytes
        let buf: Vec<u8> = vec![0x0; len as usize];

        // 2. Call env_socket_read
        let result = TcpRead {
            socket: self.socket,
            buf,
            len,
        }.await;

        if result.is_err() {
            info!("Socket read failed");
            return Err(result.err().unwrap());
        }

        // 3. If no error, return the read buffer
        return Ok(result.unwrap());
    }

    pub async fn write(&self, buf: &[u8]) -> Result<usize, LwipError> {
        // 1. Call env_socket_write
        let result = unsafe { ffi::env_socket_write(self.socket, buf.as_ptr(), buf.len() as u32) };

        if result != LwipError::Ok.to_code() {
            // TODO: Delete the socket
            info!("Socket write failed");
            return Err(LwipError::from_code(result));
        }

        // 3. If no error, return a tcp write future
        let result = TcpWrite { socket: self.socket }.await;
        if result.is_err() {
            // TODO: Delete the socket
            info!("Awaiting tcp write failed");
            return Err(result.err().unwrap());
        }

        // 4. If no error, return the number of bytes written
        return Ok(buf.len());
    }
}
