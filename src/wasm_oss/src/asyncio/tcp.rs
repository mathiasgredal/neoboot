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

pub struct TcpSocket {
    pub socket: u64,
}

impl TcpSocket {
    pub async fn connect(addr_str: &str, port: u16) -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_socket_create() };
        if socket == 0 {
            return Err(LwipError::ConnectionAborted);
        }

        let addr = ip_addr_to_u32(addr_str)?;
        let result: i32 = unsafe { ffi::env_socket_connect(socket, addr, port.into()) };

        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        // Wait for connection
        let result = TcpConnection { socket }.await;

        // If the connection attempt failed, close the socket
        if result.is_err() {
            info!("Failed to connect to {}:{}", addr_str, port);
            let close_result = unsafe { ffi::env_socket_close(socket) };
            if close_result != LwipError::Ok.to_code() {
                error!(
                    "Failed to close socket: {}",
                    LwipError::from_code(close_result)
                );
                error!("NOT IMPLEMENTED: socket closing retry functionality, leaking socket");
            }
            return Err(result.err().unwrap());
        }

        Ok(Self { socket })
    }

    // pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
    //     // Implementation for reading
    // }

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
