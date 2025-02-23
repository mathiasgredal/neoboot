use crate::{ffi, lwip_error::LwipError};
use embedded_nal_async::AddrType;
use futures::lock::Mutex;
use log::info;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    pin::Pin,
    task::{Context, Poll},
};

pub struct Dns {
    lock: Mutex<()>,
}

impl Dns {
    pub fn new() -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }
}

impl embedded_nal_async::Dns for Dns {
    type Error = LwipError;

    async fn get_host_by_name(
        &self,
        host: &str,
        _addr_type: AddrType,
    ) -> Result<IpAddr, Self::Error> {
        info!("DNS lookup: {}", host);
        // Use a guard pattern for the lock
        let _guard = self.lock.lock().await;

        // Start DNS lookup
        unsafe {
            let result = ffi::env_net_dns_lookup(host.as_ptr(), host.len() as u32);
            info!("DNS lookup result: {}", result);
            if result != LwipError::Ok.to_code() {
                info!("DNS lookup failed: {}", LwipError::from_code(result));
                return Err(LwipError::from_code(result));
            }
        }

        // Poll until we get a result
        poll_dns().await
    }

    async fn get_host_by_address(&self, _: IpAddr, _: &mut [u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}

/// Future that polls DNS resolution
async fn poll_dns() -> Result<IpAddr, LwipError> {
    struct DnsPollingFuture;

    impl Future for DnsPollingFuture {
        type Output = Result<IpAddr, LwipError>;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            unsafe { ffi::env_net_rx() };

            // Check DNS lookup status
            let status = unsafe { ffi::env_net_dns_lookup_poll() };

            match LwipError::from_code(status) {
                LwipError::Ok => {
                    let result = unsafe { ffi::env_net_dns_lookup_result() };
                    let chunks = result.to_be_bytes();
                    let ip = Ipv4Addr::new(chunks[3], chunks[2], chunks[1], chunks[0]);
                    Poll::Ready(Ok(IpAddr::V4(ip)))
                }
                LwipError::InProgress => Poll::Pending,
                err => {
                    info!("DNS lookup failed result: {}", status);
                    Poll::Ready(Err(err))
                }
            }
        }
    }

    DnsPollingFuture.await
}
