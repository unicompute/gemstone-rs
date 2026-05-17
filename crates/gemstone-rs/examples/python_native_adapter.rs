use gemstone_rs::{
    py_native::{capabilities, PyNativeConfig, PyNativeSession, PyNativeValue},
    Result,
};

fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "--dry-run") {
        let capabilities = capabilities();
        println!("gemstone-py-native adapter contract");
        println!("  contract_version: {}", capabilities.contract_version);
        println!("  threading: {}", capabilities.threading);
        println!("  operations: {}", capabilities.operations.join(", "));
        println!("  dry_run: no GemStone login attempted");
        return Ok(());
    }

    let config = PyNativeConfig::from_env()?;
    let summary = config.redacted_summary();
    println!(
        "connecting to stone={} host={} user={} password_set={}",
        summary.stone, summary.host, summary.username, summary.password_set
    );

    let mut session = PyNativeSession::login(config)?;
    assert_eq!(session.eval("3 + 4")?, PyNativeValue::SmallInt(7));

    let printed = session.perform_values(PyNativeValue::SmallInt(7), "printString", &[])?;
    let printed_oop = printed
        .raw_oop()
        .expect("printString should return a String OOP");
    assert_eq!(session.fetch_string(printed_oop)?, "7");

    let key = format!("GemStoneRsPyNative{}", std::process::id());
    session.global_put_value(&key, PyNativeValue::String("shared core".to_string()))?;
    session.commit()?;
    let stored = session.global_get(&key)?;
    assert_eq!(session.fetch_string(stored)?, "shared core");
    session.global_put_raw(&key, gemstone_rs::OOP_NIL)?;
    session.commit()?;
    session.logout()?;

    println!("gemstone-py-native adapter smoke passed");
    Ok(())
}
