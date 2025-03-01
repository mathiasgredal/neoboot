use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ffi;

pub struct Sleep {
    deadline: u64,
}

impl Sleep {
    pub fn new(duration_ms: u64) -> Self {
        let deadline = unsafe { ffi::env_now() } + duration_ms;
        Self { deadline }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = unsafe { ffi::env_now() };
        if now >= self.deadline {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
