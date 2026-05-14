// Requires a live GemStone/S stone.
//
// This scaffold shows the conservative Rust web-service shape: a fixed number
// of GemStone sessions, each owned by one dedicated worker thread, shared
// through a cloneable pool handle.
//
// Expected output includes:
//
// started GemStone worker pool with 2 workers
// eval 3 + 4 -> SmallInt(7)

use gemstone_rs::{Config, Oop, SessionWorkerPool, Value};

fn main() -> gemstone_rs::Result<()> {
    let pool = SessionWorkerPool::start(Config::from_env()?, 2)?;

    println!("started GemStone worker pool with {} workers", pool.size());

    let value = pool.eval("3 + 4")?;
    assert_eq!(value, Value::SmallInt(7));
    println!("eval 3 + 4 -> {value:?}");

    let printed = pool.perform_oop(Oop::from_smallint(7), "printString", &[])?;
    println!("perform 7 printString -> {}", pool.fetch_string(printed)?);

    pool.shutdown()?;
    Ok(())
}
