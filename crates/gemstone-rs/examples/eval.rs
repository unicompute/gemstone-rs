// Requires a live GemStone/S stone.
//
// Expected output:
//
// SmallInt(7)

use gemstone_rs::{Config, Session};

fn main() -> gemstone_rs::Result<()> {
    let config = Config::from_env()?;
    let mut session = Session::login(config)?;
    let value = session.eval("3 + 4")?;

    println!("{value:?}");

    session.logout()?;
    Ok(())
}
