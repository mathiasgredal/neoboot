use futures::lock::Mutex;
use futures::task::ArcWake;
use std::cell::RefCell;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;
use std::sync::Arc;
use std::task::RawWaker;
use std::task::RawWakerVTable;
use std::task::Waker;
use std::task::{Context, Poll};

#[cfg(feature = "executor_metrics")]
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);

struct ExecutorInner<'a> {
    scheduled: mpsc::Receiver<Arc<Task<'a>>>,
    sender: mpsc::Sender<Arc<Task<'a>>>,
    exit_flag: bool,
    exit_waker: Option<Waker>,
}

#[derive(Clone)]
pub struct Executor<'a> {
    inner: Rc<RefCell<ExecutorInner<'a>>>,
}

// SAFETY: Executor is single-threaded, so it is safe to implement Sync and Send.
unsafe impl Sync for Executor<'_> {}
unsafe impl Send for Executor<'_> {}

impl<'a> Executor<'a> {
    /// Initialize a new executor instance.
    pub fn new() -> Executor<'a> {
        let (sender, scheduled) = mpsc::channel();

        Executor {
            inner: Rc::new(RefCell::new(ExecutorInner {
                scheduled,
                sender,
                exit_flag: false,
                exit_waker: None,
            })),
        }
    }

    /// Spawn a future onto the executor instance.
    ///
    /// The given future is wrapped with the `Task` harness and pushed into the
    /// `scheduled` queue. The future will be executed when `run` is called.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'a,
    {
        Task::spawn(future, &self.inner.borrow().sender);
    }

    /// Run the executor until the exit flag is set.
    pub fn run_forever(&self) {
        loop {
            if self.inner.borrow().exit_flag {
                let task = self.inner.borrow().scheduled.try_recv();
                if task.is_err() {
                    break;
                }
                task.unwrap().poll();
            } else {
                let task = self.inner.borrow().scheduled.recv();
                task.unwrap().poll();
            }
        }
    }

    /// Exit the executor.
    pub fn exit(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.exit_flag = true;
        if let Some(waker) = inner.exit_waker.take() {
            waker.wake_by_ref();
        }
    }

    /// Get the number of active tasks.
    pub fn active_tasks(&self) -> usize {
        #[cfg(feature = "executor_metrics")]
        return ACTIVE_TASKS.load(std::sync::atomic::Ordering::Relaxed);
        0
    }

    /// An async function that waits for the executor to exit.
    pub async fn wait_for_exit(&self) {
        struct WaitForExit<'a> {
            executor: Executor<'a>,
        }

        impl Future for WaitForExit<'_> {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let executor = self.executor.clone();
                let mut inner = executor.inner.borrow_mut();

                if inner.exit_flag {
                    return Poll::Ready(());
                }

                inner.exit_waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }

        WaitForExit {
            executor: self.clone(),
        }
        .await;
    }
}

struct TaskFuture<'a> {
    future: Pin<Box<dyn Future<Output = ()> + 'a>>,
    poll: Poll<()>,
}

struct Task<'a> {
    task_future: Mutex<TaskFuture<'a>>,
    executor: mpsc::Sender<Arc<Task<'a>>>,
}

// SAFETY: Since our executor is single-threaded, we can safely implement Sync and Send for Task.
unsafe impl Sync for Task<'_> {}
unsafe impl Send for Task<'_> {}

impl ArcWake for Task<'_> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

impl<'a> TaskFuture<'a> {
    fn new(future: impl Future<Output = ()> + 'a) -> TaskFuture<'a> {
        TaskFuture {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        // Spurious wake-ups are allowed, even after a future has
        // returned `Ready`. However, polling a future which has
        // already returned `Ready` is *not* allowed. For this
        // reason we need to check that the future is still pending
        // before we call it. Failure to do so can lead to a panic.
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}

#[allow(clippy::redundant_clone)] // The clone here isn't actually redundant.
unsafe fn increase_refcount<'a, T: ArcWake + 'a>(data: *const ()) {
    // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
    let arc = mem::ManuallyDrop::new(unsafe { Arc::<T>::from_raw(data.cast::<T>()) });
    // Now increase refcount, but don't drop new refcount either
    let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
}

#[inline(always)]
unsafe fn clone_arc_raw<'a, T: ArcWake + 'a>(data: *const ()) -> RawWaker {
    unsafe { increase_refcount::<T>(data) }
    RawWaker::new(data, waker_vtable::<T>())
}

unsafe fn wake_arc_raw<'a, T: ArcWake + 'a>(data: *const ()) {
    let arc: Arc<T> = unsafe { Arc::from_raw(data.cast::<T>()) };
    ArcWake::wake(arc);
}

// used by `waker_ref`
unsafe fn wake_by_ref_arc_raw<'a, T: ArcWake + 'a>(data: *const ()) {
    // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
    let arc = mem::ManuallyDrop::new(unsafe { Arc::<T>::from_raw(data.cast::<T>()) });
    ArcWake::wake_by_ref(&arc);
}

unsafe fn drop_arc_raw<'a, T: ArcWake + 'a>(data: *const ()) {
    drop(unsafe { Arc::<T>::from_raw(data.cast::<T>()) })
}

fn waker_vtable<'a, W: ArcWake + 'a>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_arc_raw::<W>,
        wake_arc_raw::<W>,
        wake_by_ref_arc_raw::<W>,
        drop_arc_raw::<W>,
    )
}

pub fn waker<'a, W: ArcWake + 'a>(wake: Arc<W>) -> Waker {
    let ptr = Arc::into_raw(wake).cast::<()>();

    unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>())) }
}

impl<'a> Task<'a> {
    fn schedule(self: &Arc<Self>) {
        #[cfg(feature = "executor_metrics")]
        let _ = ACTIVE_TASKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = self.executor.send(self.clone());
    }

    fn poll(self: Arc<Self>) {
        #[cfg(feature = "executor_metrics")]
        if ACTIVE_TASKS.load(std::sync::atomic::Ordering::Relaxed) > 1 {
            let _ = ACTIVE_TASKS.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }

        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = waker(self.clone());
        let mut cx: Context<'_> = Context::from_waker(&waker);

        // No other thread ever tries to lock the task_future
        let mut task_future = self.task_future.try_lock().unwrap();

        // Poll the inner future
        task_future.poll(&mut cx);
    }

    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task<'a>>>)
    where
        F: Future<Output = ()> + 'a,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}
