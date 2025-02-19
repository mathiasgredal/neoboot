use log::{error, info};
use std::cell::RefCell;
use std::future::Future;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::ffi;
use crate::lwip_error::LwipError;

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
