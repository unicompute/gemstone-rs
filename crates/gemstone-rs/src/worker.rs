use crate::{Config, Oop, Result, Session, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Sender},
    Arc, Mutex,
};
use std::task::{Context, Poll, Waker};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce(&mut Session) + Send + 'static>;

enum WorkerRequest {
    Call(Job),
    Shutdown,
}

struct WorkerInner {
    sender: Sender<WorkerRequest>,
    join: Mutex<Option<JoinHandle<()>>>,
}

struct SessionWorkerPoolInner {
    workers: Vec<SessionWorker>,
    next: AtomicUsize,
}

struct WorkerFutureState<T> {
    result: Option<Result<T>>,
    waker: Option<Waker>,
}

/// Awaitable result for a call scheduled on a dedicated GemStone worker.
///
/// This future is dependency-free and does not move `Session` across threads.
/// The worker thread owns the `Session`, stores the result, and wakes the
/// async task that is polling this future.
#[must_use = "worker futures do nothing unless awaited or polled"]
pub struct SessionWorkerFuture<T> {
    state: Arc<Mutex<WorkerFutureState<T>>>,
}

impl<T> Future for SessionWorkerFuture<T> {
    type Output = Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Ok(mut state) = self.state.lock() else {
            return Poll::Ready(Err(crate::Error::WorkerStopped));
        };
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            let should_update = state
                .waker
                .as_ref()
                .is_none_or(|waker| !waker.will_wake(cx.waker()));
            if should_update {
                state.waker = Some(cx.waker().clone());
            }
            Poll::Pending
        }
    }
}

fn pending_worker_future<T>() -> SessionWorkerFuture<T> {
    SessionWorkerFuture {
        state: Arc::new(Mutex::new(WorkerFutureState {
            result: None,
            waker: None,
        })),
    }
}

fn ready_worker_future<T>(result: Result<T>) -> SessionWorkerFuture<T> {
    SessionWorkerFuture {
        state: Arc::new(Mutex::new(WorkerFutureState {
            result: Some(result),
            waker: None,
        })),
    }
}

fn complete_worker_future<T>(state: &Arc<Mutex<WorkerFutureState<T>>>, result: Result<T>) {
    let Ok(mut state) = state.lock() else {
        return;
    };
    state.result = Some(result);
    if let Some(waker) = state.waker.take() {
        waker.wake();
    }
}

/// Dedicated GemStone session worker.
///
/// `Session` is deliberately not `Send` or `Sync`, because GCI thread-safety
/// should be treated conservatively. `SessionWorker` creates and owns the
/// session on one dedicated thread, then serializes requests onto that thread.
/// Clone this handle or wrap it in application state when a web framework needs
/// shared access to one session lane.
#[derive(Clone)]
pub struct SessionWorker {
    inner: Arc<WorkerInner>,
}

impl SessionWorker {
    /// Start a worker and log in on the worker thread before returning.
    pub fn start(config: Config) -> Result<Self> {
        let (sender, receiver) = mpsc::channel::<WorkerRequest>();
        let (ready_sender, ready_receiver) = mpsc::channel::<Result<()>>();

        let join = thread::spawn(move || {
            let mut session = match Session::login(config) {
                Ok(session) => {
                    let _ = ready_sender.send(Ok(()));
                    session
                }
                Err(err) => {
                    let _ = ready_sender.send(Err(err));
                    return;
                }
            };

            while let Ok(request) = receiver.recv() {
                match request {
                    WorkerRequest::Call(job) => job(&mut session),
                    WorkerRequest::Shutdown => break,
                }
            }

            let _ = session.logout();
        });

        match ready_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                inner: Arc::new(WorkerInner {
                    sender,
                    join: Mutex::new(Some(join)),
                }),
            }),
            Ok(Err(err)) => {
                let _ = join.join();
                Err(err)
            }
            Err(_) => {
                let _ = join.join();
                Err(crate::Error::WorkerStopped)
            }
        }
    }

    /// Run a custom closure on the worker-owned `Session`.
    pub fn call<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> Result<T>
    where
        T: Send + 'static,
    {
        let (response_sender, response_receiver) = mpsc::channel();
        self.inner
            .sender
            .send(WorkerRequest::Call(Box::new(move |session| {
                let _ = response_sender.send(body(session));
            })))
            .map_err(|_| crate::Error::WorkerStopped)?;
        response_receiver
            .recv()
            .map_err(|_| crate::Error::WorkerStopped)?
    }

    /// Schedule a custom closure on the worker-owned `Session` and return an
    /// awaitable future.
    ///
    /// This is the async-runtime friendly form of [`Self::call`]. It queues the
    /// operation on the same dedicated worker thread but does not block the
    /// async task while waiting for the result.
    pub fn call_async<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> SessionWorkerFuture<T>
    where
        T: Send + 'static,
    {
        let future = pending_worker_future();
        let state = Arc::clone(&future.state);
        let completion_state = Arc::clone(&state);
        if self
            .inner
            .sender
            .send(WorkerRequest::Call(Box::new(move |session| {
                complete_worker_future(&completion_state, body(session));
            })))
            .is_err()
        {
            complete_worker_future(&state, Err(crate::Error::WorkerStopped));
        }
        future
    }

    pub fn eval(&self, source: impl Into<String>) -> Result<Value> {
        let source = source.into();
        self.call(move |session| session.eval(&source))
    }

    pub fn eval_async(&self, source: impl Into<String>) -> SessionWorkerFuture<Value> {
        let source = source.into();
        self.call_async(move |session| session.eval(&source))
    }

    pub fn eval_oop(&self, source: impl Into<String>) -> Result<Oop> {
        let source = source.into();
        self.call(move |session| session.eval_oop(&source))
    }

    pub fn eval_oop_async(&self, source: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let source = source.into();
        self.call_async(move |session| session.eval_oop(&source))
    }

    pub fn execute(&self, source: impl Into<String>) -> Result<Oop> {
        let source = source.into();
        self.call(move |session| session.execute(&source))
    }

    pub fn execute_async(&self, source: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let source = source.into();
        self.call_async(move |session| session.execute(&source))
    }

    pub fn resolve(&self, name: impl Into<String>) -> Result<Oop> {
        let name = name.into();
        self.call(move |session| session.resolve(&name))
    }

    pub fn resolve_async(&self, name: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let name = name.into();
        self.call_async(move |session| session.resolve(&name))
    }

    pub fn perform(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> Result<Value> {
        let selector = selector.into();
        let args = args.to_vec();
        self.call(move |session| session.perform(receiver, &selector, &args))
    }

    pub fn perform_async(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> SessionWorkerFuture<Value> {
        let selector = selector.into();
        let args = args.to_vec();
        self.call_async(move |session| session.perform(receiver, &selector, &args))
    }

    pub fn perform_oop(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> Result<Oop> {
        let selector = selector.into();
        let args = args.to_vec();
        self.call(move |session| session.perform_oop(receiver, &selector, &args))
    }

    pub fn perform_oop_async(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> SessionWorkerFuture<Oop> {
        let selector = selector.into();
        let args = args.to_vec();
        self.call_async(move |session| session.perform_oop(receiver, &selector, &args))
    }

    pub fn new_string(&self, value: impl Into<String>) -> Result<Oop> {
        let value = value.into();
        self.call(move |session| session.new_string(&value))
    }

    pub fn new_string_async(&self, value: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let value = value.into();
        self.call_async(move |session| session.new_string(&value))
    }

    pub fn new_symbol(&self, value: impl Into<String>) -> Result<Oop> {
        let value = value.into();
        self.call(move |session| session.new_symbol(&value))
    }

    pub fn new_symbol_async(&self, value: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let value = value.into();
        self.call_async(move |session| session.new_symbol(&value))
    }

    pub fn fetch_string(&self, oop: Oop) -> Result<String> {
        self.call(move |session| session.fetch_string(oop))
    }

    pub fn fetch_string_async(&self, oop: Oop) -> SessionWorkerFuture<String> {
        self.call_async(move |session| session.fetch_string(oop))
    }

    pub fn global_get(&self, symbol_name: impl Into<String>) -> Result<Oop> {
        let symbol_name = symbol_name.into();
        self.call(move |session| session.global_get(&symbol_name))
    }

    pub fn global_get_async(&self, symbol_name: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let symbol_name = symbol_name.into();
        self.call_async(move |session| session.global_get(&symbol_name))
    }

    pub fn global_put(&self, symbol_name: impl Into<String>, value: Oop) -> Result<()> {
        let symbol_name = symbol_name.into();
        self.call(move |session| session.global_put(&symbol_name, value))
    }

    pub fn global_put_async(
        &self,
        symbol_name: impl Into<String>,
        value: Oop,
    ) -> SessionWorkerFuture<()> {
        let symbol_name = symbol_name.into();
        self.call_async(move |session| session.global_put(&symbol_name, value))
    }

    pub fn commit(&self) -> Result<()> {
        self.call(Session::commit)
    }

    pub fn commit_async(&self) -> SessionWorkerFuture<()> {
        self.call_async(Session::commit)
    }

    pub fn abort(&self) -> Result<()> {
        self.call(Session::abort)
    }

    pub fn abort_async(&self) -> SessionWorkerFuture<()> {
        self.call_async(Session::abort)
    }

    pub fn needs_commit(&self) -> Result<bool> {
        self.call(Session::needs_commit)
    }

    pub fn needs_commit_async(&self) -> SessionWorkerFuture<bool> {
        self.call_async(Session::needs_commit)
    }

    pub fn transaction<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> Result<T>
    where
        T: Send + 'static,
    {
        self.call(move |session| session.transaction(body))
    }

    pub fn transaction_async<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> SessionWorkerFuture<T>
    where
        T: Send + 'static,
    {
        self.call_async(move |session| session.transaction(body))
    }

    /// Stop the worker and join the owned thread.
    ///
    /// Other cloned handles become stopped after this succeeds.
    pub fn shutdown(self) -> Result<()> {
        let _ = self.inner.sender.send(WorkerRequest::Shutdown);
        self.join_worker()
    }

    fn join_worker(&self) -> Result<()> {
        let Some(join) = self
            .inner
            .join
            .lock()
            .map_err(|_| crate::Error::WorkerStopped)?
            .take()
        else {
            return Ok(());
        };
        join.join().map_err(|_| crate::Error::WorkerPanicked)
    }
}

impl Drop for WorkerInner {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerRequest::Shutdown);
        if let Ok(mut join) = self.join.lock() {
            if let Some(join) = join.take() {
                let _ = join.join();
            }
        }
    }
}

/// Bounded pool of dedicated GemStone session workers.
///
/// A pool starts `size` independent `SessionWorker` instances and chooses a
/// worker in round-robin order for each call. This is the conservative web
/// service shape: every underlying `Session` still lives on one dedicated
/// thread, while HTTP handlers or background jobs can share a cloneable pool
/// handle.
#[derive(Clone)]
pub struct SessionWorkerPool {
    inner: Arc<SessionWorkerPoolInner>,
}

impl SessionWorkerPool {
    /// Start a fixed-size worker pool.
    ///
    /// `size` must be greater than zero. If any worker fails to log in, the
    /// already-started workers are dropped and the login error is returned.
    pub fn start(config: Config, size: usize) -> Result<Self> {
        if size == 0 {
            return Err(crate::Error::MissingConfig("worker_count"));
        }

        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            workers.push(SessionWorker::start(config.clone())?);
        }

        Ok(Self {
            inner: Arc::new(SessionWorkerPoolInner {
                workers,
                next: AtomicUsize::new(0),
            }),
        })
    }

    pub fn size(&self) -> usize {
        self.inner.workers.len()
    }

    /// Return the next worker in round-robin order.
    pub fn worker(&self) -> SessionWorker {
        let index = self.inner.next.fetch_add(1, Ordering::Relaxed) % self.inner.workers.len();
        self.inner.workers[index].clone()
    }

    /// Run a custom closure on one worker-owned `Session`.
    pub fn call<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> Result<T>
    where
        T: Send + 'static,
    {
        self.worker().call(body)
    }

    /// Schedule a custom closure on the next worker and return an awaitable
    /// future.
    pub fn call_async<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> SessionWorkerFuture<T>
    where
        T: Send + 'static,
    {
        if self.inner.workers.is_empty() {
            ready_worker_future(Err(crate::Error::MissingConfig("worker_count")))
        } else {
            self.worker().call_async(body)
        }
    }

    pub fn eval(&self, source: impl Into<String>) -> Result<Value> {
        self.worker().eval(source)
    }

    pub fn eval_async(&self, source: impl Into<String>) -> SessionWorkerFuture<Value> {
        self.call_async({
            let source = source.into();
            move |session| session.eval(&source)
        })
    }

    pub fn eval_oop(&self, source: impl Into<String>) -> Result<Oop> {
        self.worker().eval_oop(source)
    }

    pub fn eval_oop_async(&self, source: impl Into<String>) -> SessionWorkerFuture<Oop> {
        self.call_async({
            let source = source.into();
            move |session| session.eval_oop(&source)
        })
    }

    pub fn execute(&self, source: impl Into<String>) -> Result<Oop> {
        self.worker().execute(source)
    }

    pub fn execute_async(&self, source: impl Into<String>) -> SessionWorkerFuture<Oop> {
        self.call_async({
            let source = source.into();
            move |session| session.execute(&source)
        })
    }

    pub fn resolve(&self, name: impl Into<String>) -> Result<Oop> {
        self.worker().resolve(name)
    }

    pub fn resolve_async(&self, name: impl Into<String>) -> SessionWorkerFuture<Oop> {
        self.call_async({
            let name = name.into();
            move |session| session.resolve(&name)
        })
    }

    pub fn perform(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> Result<Value> {
        self.worker().perform(receiver, selector, args)
    }

    pub fn perform_async(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> SessionWorkerFuture<Value> {
        let selector = selector.into();
        let args = args.to_vec();
        self.call_async(move |session| session.perform(receiver, &selector, &args))
    }

    pub fn perform_oop(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> Result<Oop> {
        self.worker().perform_oop(receiver, selector, args)
    }

    pub fn perform_oop_async(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> SessionWorkerFuture<Oop> {
        let selector = selector.into();
        let args = args.to_vec();
        self.call_async(move |session| session.perform_oop(receiver, &selector, &args))
    }

    pub fn new_string(&self, value: impl Into<String>) -> Result<Oop> {
        self.worker().new_string(value)
    }

    pub fn new_string_async(&self, value: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let value = value.into();
        self.call_async(move |session| session.new_string(&value))
    }

    pub fn new_symbol(&self, value: impl Into<String>) -> Result<Oop> {
        self.worker().new_symbol(value)
    }

    pub fn new_symbol_async(&self, value: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let value = value.into();
        self.call_async(move |session| session.new_symbol(&value))
    }

    pub fn fetch_string(&self, oop: Oop) -> Result<String> {
        self.worker().fetch_string(oop)
    }

    pub fn fetch_string_async(&self, oop: Oop) -> SessionWorkerFuture<String> {
        self.call_async(move |session| session.fetch_string(oop))
    }

    pub fn global_get(&self, symbol_name: impl Into<String>) -> Result<Oop> {
        self.worker().global_get(symbol_name)
    }

    pub fn global_get_async(&self, symbol_name: impl Into<String>) -> SessionWorkerFuture<Oop> {
        let symbol_name = symbol_name.into();
        self.call_async(move |session| session.global_get(&symbol_name))
    }

    pub fn global_put(&self, symbol_name: impl Into<String>, value: Oop) -> Result<()> {
        self.worker().global_put(symbol_name, value)
    }

    pub fn global_put_async(
        &self,
        symbol_name: impl Into<String>,
        value: Oop,
    ) -> SessionWorkerFuture<()> {
        let symbol_name = symbol_name.into();
        self.call_async(move |session| session.global_put(&symbol_name, value))
    }

    pub fn commit(&self) -> Result<()> {
        self.worker().commit()
    }

    pub fn commit_async(&self) -> SessionWorkerFuture<()> {
        self.call_async(Session::commit)
    }

    pub fn abort(&self) -> Result<()> {
        self.worker().abort()
    }

    pub fn abort_async(&self) -> SessionWorkerFuture<()> {
        self.call_async(Session::abort)
    }

    pub fn needs_commit(&self) -> Result<bool> {
        self.worker().needs_commit()
    }

    pub fn needs_commit_async(&self) -> SessionWorkerFuture<bool> {
        self.call_async(Session::needs_commit)
    }

    pub fn transaction<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> Result<T>
    where
        T: Send + 'static,
    {
        self.worker().transaction(body)
    }

    pub fn transaction_async<T>(
        &self,
        body: impl FnOnce(&mut Session) -> Result<T> + Send + 'static,
    ) -> SessionWorkerFuture<T>
    where
        T: Send + 'static,
    {
        self.call_async(move |session| session.transaction(body))
    }

    /// Stop every worker and join each owned thread.
    ///
    /// Other cloned pool handles become stopped after this succeeds.
    pub fn shutdown(self) -> Result<()> {
        let mut first_error = None;
        for worker in &self.inner.workers {
            if let Err(err) = worker.clone().shutdown() {
                if first_error.is_none() {
                    first_error = Some(err);
                }
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}
