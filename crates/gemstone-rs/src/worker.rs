use crate::{Config, Oop, Result, Session, Value};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Sender},
    Arc, Mutex,
};
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

    pub fn eval(&self, source: impl Into<String>) -> Result<Value> {
        let source = source.into();
        self.call(move |session| session.eval(&source))
    }

    pub fn eval_oop(&self, source: impl Into<String>) -> Result<Oop> {
        let source = source.into();
        self.call(move |session| session.eval_oop(&source))
    }

    pub fn execute(&self, source: impl Into<String>) -> Result<Oop> {
        let source = source.into();
        self.call(move |session| session.execute(&source))
    }

    pub fn resolve(&self, name: impl Into<String>) -> Result<Oop> {
        let name = name.into();
        self.call(move |session| session.resolve(&name))
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

    pub fn new_string(&self, value: impl Into<String>) -> Result<Oop> {
        let value = value.into();
        self.call(move |session| session.new_string(&value))
    }

    pub fn fetch_string(&self, oop: Oop) -> Result<String> {
        self.call(move |session| session.fetch_string(oop))
    }

    pub fn global_get(&self, symbol_name: impl Into<String>) -> Result<Oop> {
        let symbol_name = symbol_name.into();
        self.call(move |session| session.global_get(&symbol_name))
    }

    pub fn global_put(&self, symbol_name: impl Into<String>, value: Oop) -> Result<()> {
        let symbol_name = symbol_name.into();
        self.call(move |session| session.global_put(&symbol_name, value))
    }

    pub fn commit(&self) -> Result<()> {
        self.call(Session::commit)
    }

    pub fn abort(&self) -> Result<()> {
        self.call(Session::abort)
    }

    pub fn needs_commit(&self) -> Result<bool> {
        self.call(Session::needs_commit)
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

    pub fn eval(&self, source: impl Into<String>) -> Result<Value> {
        self.worker().eval(source)
    }

    pub fn eval_oop(&self, source: impl Into<String>) -> Result<Oop> {
        self.worker().eval_oop(source)
    }

    pub fn execute(&self, source: impl Into<String>) -> Result<Oop> {
        self.worker().execute(source)
    }

    pub fn resolve(&self, name: impl Into<String>) -> Result<Oop> {
        self.worker().resolve(name)
    }

    pub fn perform(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> Result<Value> {
        self.worker().perform(receiver, selector, args)
    }

    pub fn perform_oop(
        &self,
        receiver: Oop,
        selector: impl Into<String>,
        args: &[Oop],
    ) -> Result<Oop> {
        self.worker().perform_oop(receiver, selector, args)
    }

    pub fn new_string(&self, value: impl Into<String>) -> Result<Oop> {
        self.worker().new_string(value)
    }

    pub fn fetch_string(&self, oop: Oop) -> Result<String> {
        self.worker().fetch_string(oop)
    }

    pub fn global_get(&self, symbol_name: impl Into<String>) -> Result<Oop> {
        self.worker().global_get(symbol_name)
    }

    pub fn global_put(&self, symbol_name: impl Into<String>, value: Oop) -> Result<()> {
        self.worker().global_put(symbol_name, value)
    }

    pub fn commit(&self) -> Result<()> {
        self.worker().commit()
    }

    pub fn abort(&self) -> Result<()> {
        self.worker().abort()
    }

    pub fn needs_commit(&self) -> Result<bool> {
        self.worker().needs_commit()
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
