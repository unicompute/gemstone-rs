// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// GemStone eval ok: SmallInt(7)
// GemStoneRsQuickstart: hello from gemstone-rs quickstart

use gemstone_rs::{Config, Oop, Session, Value};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;

    let value = session.eval("3 + 4")?;
    assert_eq!(value, Value::SmallInt(7));
    println!("GemStone eval ok: {value:?}");

    let key = "GemStoneRsQuickstart";
    let text = session.new_string("hello from gemstone-rs quickstart")?;
    session.global_put(key, text)?;

    let stored = session.global_get(key)?;
    println!("{key}: {}", session.fetch_string(stored)?);

    session.global_put(key, Oop::NIL)?;
    session.logout()?;
    Ok(())
}
