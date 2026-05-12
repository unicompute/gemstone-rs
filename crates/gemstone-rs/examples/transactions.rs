// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// committed value: committed by gemstone-rs
// abort path returned error: true

use gemstone_rs::{Config, Error, Oop, Session};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;

    let committed_key = "GemStoneRsTransactionCommitted";
    session.transaction(|session| {
        let value = session.new_string("committed by gemstone-rs")?;
        session.global_put(committed_key, value)
    })?;

    let committed = session.global_get(committed_key)?;
    println!("committed value: {}", session.fetch_string(committed)?);

    let aborted_key = "GemStoneRsTransactionAborted";
    let aborted: gemstone_rs::Result<()> = session.transaction(|session| {
        let value = session.new_string("this write should abort")?;
        session.global_put(aborted_key, value)?;
        Err(Error::IllegalOop {
            operation: "intentional example abort",
        })
    });

    println!("abort path returned error: {}", aborted.is_err());
    session.global_put(committed_key, Oop::NIL)?;
    Ok(())
}
