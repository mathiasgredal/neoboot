use crate::asyncio::sleep_ms;
use crate::executor::Executor;
use futures::future::Either;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

// Custom timeout controller that can be reset
pub struct TimeoutController {
    duration_ms: u64,
    waker: Option<Waker>,
    expired: bool,
}

impl TimeoutController {
    pub fn new(duration_ms: u64) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            duration_ms,
            waker: None,
            expired: false,
        }))
    }

    fn reset(&mut self) {
        self.expired = false;
    }

    fn set_expired(&mut self) {
        self.expired = true;
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

// Future that completes when the timeout expires
struct TimeoutFuture {
    controller: Arc<Mutex<TimeoutController>>,
}

impl Future for TimeoutFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut controller = self.controller.lock().unwrap();
        if controller.expired {
            Poll::Ready(())
        } else {
            controller.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

// Sets up a timeout task that can be reset
pub async fn setup_timeout_task<'b>(
    executor: &Executor<'b>,
    controller: Arc<Mutex<TimeoutController>>,
) {
    let duration_ms = controller.lock().unwrap().duration_ms;

    executor.spawn(async move {
        loop {
            sleep_ms(duration_ms).await;

            let mut controller_guard = controller.lock().unwrap();
            controller_guard.set_expired();

            // Wait until someone resets the timer
            while controller_guard.expired {
                drop(controller_guard);
                sleep_ms(10).await; // Small delay to prevent busy waiting
                controller_guard = controller.lock().unwrap();
            }
        }
    });
}

// Custom timeout implementation using our controller
pub async fn timeout_with_controller<T>(
    controller: Arc<Mutex<TimeoutController>>,
    future: impl Future<Output = T>,
) -> Result<T, &'static str> {
    // Reset the timer before starting
    {
        let mut controller_guard = controller.lock().unwrap();
        controller_guard.reset();
    }

    let timeout_future = TimeoutFuture {
        controller: controller.clone(),
    };

    match futures::future::select(Box::pin(timeout_future), Box::pin(future)).await {
        Either::Left((_, _)) => Err("Operation timed out"),
        Either::Right((value, _)) => Ok(value),
    }
}
