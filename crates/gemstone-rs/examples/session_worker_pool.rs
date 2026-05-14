use gemstone_rs::{Config, Oop, SessionWorkerPool, Value};

fn main() -> gemstone_rs::Result<()> {
    let pool = SessionWorkerPool::start(Config::from_env()?, 2)?;

    println!("started GemStone worker pool with {} workers", pool.size());

    let first = pool.eval("3 + 4")?;
    assert_eq!(first, Value::SmallInt(7));
    println!("eval 3 + 4 -> {first:?}");

    let second = pool.eval("40 + 2")?;
    assert_eq!(second, Value::SmallInt(42));
    println!("eval 40 + 2 -> {second:?}");

    let seven = Oop::from_smallint(7);
    let printed = pool.perform_oop(seven, "printString", &[])?;
    println!("perform 7 printString -> {}", pool.fetch_string(printed)?);

    pool.transaction(|session| {
        let value = session.new_string("hello from SessionWorkerPool")?;
        session.global_put("GemStoneRsWorkerPoolExample", value)
    })?;
    let stored = pool.global_get("GemStoneRsWorkerPoolExample")?;
    println!("global round trip -> {}", pool.fetch_string(stored)?);

    pool.shutdown()?;
    Ok(())
}
