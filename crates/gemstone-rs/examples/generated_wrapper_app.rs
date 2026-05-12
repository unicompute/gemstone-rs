// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// generated wrapper printString: 7

#[allow(dead_code)]
#[path = "../../../examples/codegen/generated/gemstone_wrappers.rs"]
mod gemstone_wrappers;

use gemstone_rs::{Config, Session};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let oop = session.smallint_oop(7);
    let mut object = gemstone_wrappers::Object::from_oop(&mut session, oop);

    let printed = object.print_string()?;
    assert_eq!(printed, "7");
    println!("generated wrapper printString: {printed}");

    Ok(())
}
