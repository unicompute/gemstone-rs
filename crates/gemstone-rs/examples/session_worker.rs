use gemstone_rs::{Config, Oop, SessionWorker, Value};

fn main() -> gemstone_rs::Result<()> {
    let worker = SessionWorker::start(Config::from_env()?)?;

    let value = worker.eval("3 + 4")?;
    assert_eq!(value, Value::SmallInt(7));
    println!("eval 3 + 4 -> {value:?}");

    let seven = Oop::from_smallint(7);
    let printed = worker.perform_oop(seven, "printString", &[])?;
    println!("perform 7 printString -> {}", worker.fetch_string(printed)?);

    worker.transaction(|session| {
        let value = session.new_string("hello from SessionWorker")?;
        session.global_put("GemStoneRsWorkerExample", value)
    })?;
    let stored = worker.global_get("GemStoneRsWorkerExample")?;
    println!("global round trip -> {}", worker.fetch_string(stored)?);

    worker.shutdown()?;
    Ok(())
}
