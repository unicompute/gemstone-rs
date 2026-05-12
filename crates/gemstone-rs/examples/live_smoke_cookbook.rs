// Requires a live GemStone/S stone.
//
// This example mirrors the live CI smoke checks in one readable command:
//
//   cargo run -p gemstone-rs --example live_smoke_cookbook
//
// Expected output includes:
//
// login ok
// eval ok: SmallInt(7)
// global round-trip ok
// perform ok: 7
// transaction commit/abort ok

use gemstone_rs::{Config, Oop, Session, Value};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    println!("login ok: session {}", session.session_id());

    let value = session.eval("3 + 4")?;
    assert_eq!(value, Value::SmallInt(7));
    println!("eval ok: {value:?}");

    let suffix = std::process::id();
    let key = format!("GemStoneRsLiveSmokeCookbook{suffix}");
    let text = session.new_string("hello from live smoke")?;
    session.global_put(&key, text)?;
    session.commit()?;
    let stored = session.global_get(&key)?;
    assert_eq!(session.fetch_string(stored)?, "hello from live smoke");
    println!("global round-trip ok");

    let printed = session.perform_oop(session.smallint_oop(7), "printString", &[])?;
    assert_eq!(session.fetch_string(printed)?, "7");
    println!("perform ok: 7");

    let committed_key = format!("GemStoneRsLiveSmokeCommitted{suffix}");
    let aborted_key = format!("GemStoneRsLiveSmokeAborted{suffix}");
    session.transaction(|tx| {
        let value = tx.new_string("committed")?;
        tx.global_put(&committed_key, value)
    })?;
    let committed = session.global_get(&committed_key)?;
    assert_eq!(session.fetch_string(committed)?, "committed");

    let value = session.new_string("aborted")?;
    session.global_put(&aborted_key, value)?;
    session.abort()?;
    assert_eq!(
        session.global_get(&aborted_key).unwrap_or(Oop::NIL),
        Oop::NIL
    );
    println!("transaction commit/abort ok");

    session.global_put(&key, Oop::NIL)?;
    session.global_put(&committed_key, Oop::NIL)?;
    session.commit()?;
    session.logout()?;
    Ok(())
}
