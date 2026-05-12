// Requires a live GemStone/S stone.
//
// Expected output includes explicit OOP and value conversions for small
// integers, booleans, strings, symbols, and an export-set handle.

use gemstone_rs::{Config, Session, Value};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;

    let seven = session.value_to_oop(&Value::SmallInt(7))?;
    let printed = session.perform_oop(seven, "printString", &[])?;
    println!(
        "small integer printString: {}",
        session.fetch_string(printed)?
    );

    let text = session.new_string("retained by gemstone-rs")?;
    {
        let handle = session.retain_oop(text)?;
        println!("retained string OOP: {}", handle.oop().raw());
        handle.release()?;
    }

    let symbol = session.new_symbol("GemStoneRsExampleSymbol")?;
    println!("new symbol OOP: {}", symbol.raw());
    Ok(())
}
