// Requires a live GemStone/S stone.
//
// Expected output includes dictionary, protocol, method, and source sections:
//
// Dictionaries:
// First Object protocols:
// First Object methods:
// Object>>printString source:

use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    Config, Session,
};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let mut browser = Browser::new(&mut session);

    println!("Dictionaries:");
    for dictionary in browser.dictionaries()?.into_iter().take(10) {
        println!("  {dictionary}");
    }

    println!("\nFirst Object protocols:");
    for protocol in browser.protocols("Object", false, "")?.into_iter().take(10) {
        println!("  {protocol}");
    }

    println!("\nFirst Object methods:");
    for selector in browser
        .methods("Object", ALL_PROTOCOLS, false, "")?
        .into_iter()
        .take(10)
    {
        println!("  {selector}");
    }

    println!("\nObject>>printString source:");
    println!("{}", browser.source("Object", "printString", false, "")?);
    Ok(())
}
