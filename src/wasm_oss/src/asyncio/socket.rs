use futures_lite::future::yield_now;
use log::{error, info};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::ffi;
use crate::lwip_error::LwipError;

pub struct SocketInner {
    pub socket: i32,
}

impl Drop for SocketInner {
    fn drop(&mut self) {
        info!("Closing socket: {}", self.socket);
        let result = unsafe { ffi::env_net_socket_free(self.socket) };
        if result != LwipError::Ok.to_code() {
            error!("Failed to close socket: {}", LwipError::from_code(result));
        }
    }
}

#[derive(Clone)]
pub struct Socket {
    pub inner: Rc<RefCell<SocketInner>>,
}

impl Socket {
    pub async fn read(&self, len: u16) -> Result<Vec<u8>, LwipError> {
        struct Read {
            socket: Socket,
            buf: Option<Vec<u8>>,
        }

        impl Future for Read {
            type Output = Result<Vec<u8>, LwipError>;

            fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                // SAFETY: FFI call to network stack
                unsafe { ffi::env_net_rx() };

                let buf = self
                    .buf
                    .as_ref()
                    .expect("Buffer should exist during polling");

                // SAFETY: We maintain exclusive control of the buffer until completion
                let result = unsafe {
                    ffi::env_net_socket_read(
                        self.socket.inner.borrow().socket,
                        buf.as_ptr(),
                        buf.len() as u32,
                    )
                };

                if result == LwipError::WouldBlock.to_code() {
                    return Poll::Pending;
                }

                let mut buf = self
                    .buf
                    .take()
                    .expect("Buffer should exist for final result");
                if result >= 0 {
                    let bytes_read = result as usize;
                    buf.truncate(bytes_read);
                    Poll::Ready(Ok(buf))
                } else {
                    Poll::Ready(Err(LwipError::from_code(result)))
                }
            }
        }

        yield_now().await;

        let buffer = vec![0u8; len as usize];

        Read {
            socket: self.clone(),
            buf: Some(buffer),
        }
        .await
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
        let result = unsafe {
            ffi::env_net_socket_write(self.inner.borrow().socket, buf.as_ptr(), buf.len() as u32)
        };

        if result != LwipError::Ok.to_code() {
            let err = LwipError::from_code(result);
            info!("Error initial writing: {:?}", err);
            return Err(err);
        }

        // 3. If no error, return a tcp write future
        let result = Write {
            socket: self.inner.borrow().socket,
        }
        .await;

        if result.is_err() {
            return Err(result.err().unwrap());
        }

        // 4. If no error, return the number of bytes written
        return Ok(buf.len());
    }
}
