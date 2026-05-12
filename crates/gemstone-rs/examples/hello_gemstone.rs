// Requires a live GemStone/S stone.
//
// Expected output includes a session id and:
//
// 3 + 4 => SmallInt(7)

use gemstone_rs::{Config, Session};

fn main() -> gemstone_rs::Result<()> {
    let config = Config::from_env()?;
    println!("stone: {}", config.stone);
    println!("host: {}", config.host);
    println!("netldi: {}", config.netldi);
    println!("username: {}", config.username);

    let mut session = Session::login(config)?;
    println!("session id: {}", session.session_id());
    println!("3 + 4 => {:?}", session.eval("3 + 4")?);
    session.logout()?;
    Ok(())
}
