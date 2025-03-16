use futures::lock::Mutex;
use futures::task;
use futures::task::ArcWake;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::task::{Context, Poll};

struct ExecutorInner {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

#[derive(Clone)]
pub struct Executor {
    inner: Rc<RefCell<ExecutorInner>>,
    exit_flag: Rc<RefCell<bool>>,
}

unsafe impl Sync for Executor {}
unsafe impl Send for Executor {}

impl Executor {
    /// Initialize a new executor instance.
    pub fn new() -> Executor {
        let (sender, scheduled) = mpsc::channel();

        Executor {
            inner: Rc::new(RefCell::new(ExecutorInner { scheduled, sender })),
            exit_flag: Rc::new(RefCell::new(false)),
        }
    }

    /// Spawn a future onto the executor instance.
    ///
    /// The given future is wrapped with the `Task` harness and pushed into the
    /// `scheduled` queue. The future will be executed when `run` is called.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        Task::spawn(future, &self.inner.borrow().sender);
    }

    /// Run the executor until the exit flag is set.
    pub fn run_forever(&self) {
        while let Ok(task) = self.inner.borrow().scheduled.recv() {
            task.poll();

            if *self.exit_flag.borrow() {
                break;
            }
        }
    }

    /// Exit the executor.
    pub fn exit(&mut self) {
        let mut exit_flag = self.exit_flag.borrow_mut();
        *exit_flag = true;
    }
}

struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()>>>,
    poll: Poll<()>,
}

struct Task {
    task_future: Mutex<TaskFuture>,
    executor: mpsc::Sender<Arc<Task>>,
}

// SAFETY: Since our executor is single-threaded, we can safely implement Sync and Send for Task.
unsafe impl Sync for Task {}
unsafe impl Send for Task {}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

impl TaskFuture {
    fn new(future: impl Future<Output = ()> + 'static) -> TaskFuture {
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

impl Task {
    fn schedule(self: &Arc<Self>) {
        let _ = self.executor.send(self.clone());
    }

    fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

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
    fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + 'static,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}
