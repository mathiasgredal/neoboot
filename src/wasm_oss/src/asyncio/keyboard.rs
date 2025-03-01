use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::ffi;

pub struct KeyPress;

impl Future for KeyPress {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let key = ffi::env_key_pressed();
            if key != -1 {
                Poll::Ready(key)
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
