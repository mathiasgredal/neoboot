use crate::{errors::lwip_error::LwipError, ffi, utils::ip_addr_to_u32};
use futures::lock::Mutex;
use log::info;
use once_cell::sync::Lazy;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr},
    pin::Pin,
    task::{Context, Poll},
};

// default dns server
const DEFAULT_DNS_SERVER: &str = "8.8.8.8";

pub static GLOBAL_DNS_RESOLVER: Lazy<Dns> = Lazy::new(|| {
    unsafe { ffi::env_net_dns_set_server(ip_addr_to_u32(DEFAULT_DNS_SERVER).unwrap()) };
    Dns::new()
});

pub struct Dns {
    lock: Mutex<()>,
}

impl Dns {
    fn new() -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }
}

impl Dns {
    pub async fn get_host_by_name(&self, host: &str) -> Result<IpAddr, LwipError> {
        let _guard = self.lock.lock().await;

        let result = unsafe { ffi::env_net_dns_lookup(host.as_ptr(), host.len() as u32) };

        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        poll_dns().await
    }
}

/// Future that polls DNS resolution
async fn poll_dns() -> Result<IpAddr, LwipError> {
    struct DnsPollingFuture;

    impl Future for DnsPollingFuture {
        type Output = Result<IpAddr, LwipError>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            unsafe { ffi::env_net_rx() };

            let status = unsafe { ffi::env_net_dns_lookup_poll() };

            match LwipError::from_code(status) {
                LwipError::Ok => {
                    let result = unsafe { ffi::env_net_dns_lookup_result() };
                    let chunks = result.to_be_bytes();
                    let ip = Ipv4Addr::new(chunks[3], chunks[2], chunks[1], chunks[0]);
                    info!("DNS lookup result: {}", ip);
                    Poll::Ready(Ok(IpAddr::V4(ip)))
                }
                LwipError::InProgress => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                err => {
                    info!("DNS lookup failed result: {}", status);
                    Poll::Ready(Err(err))
                }
            }
        }
    }

    DnsPollingFuture.await
}
