use gemstone_rs::{Config, Oop, Result, SessionWorkerPool, Value};
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "--dry-run") {
        println!("gemstone-rs async worker example");
        println!("  cargo run -p gemstone-rs --example async_worker");
        println!("  requires: GS_USERNAME, GS_PASSWORD, and a reachable GemStone/S stone");
        return Ok(());
    }
    block_on(run())
}

async fn run() -> Result<()> {
    let pool = SessionWorkerPool::start(Config::from_env()?, 2)?;
    let value = pool.eval_async("3 + 4").await?;
    println!("async eval: {value:?}");
    assert_eq!(value, Value::SmallInt(7));

    let printed = pool
        .perform_oop_async(Oop::from_smallint(7), "printString", &[])
        .await?;
    let text = pool.fetch_string_async(printed).await?;
    println!("async perform printString: {text}");
    assert_eq!(text, "7");

    pool.shutdown()?;
    Ok(())
}

struct ThreadWake;

impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {}
}

fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(ThreadWake));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match Future::poll(future.as_mut(), &mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::sleep(Duration::from_millis(1)),
        }
    }
}
