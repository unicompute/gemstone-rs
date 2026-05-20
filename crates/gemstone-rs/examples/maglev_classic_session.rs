// Requires a live GemStone/S stone.
//
// Rust equivalent of the classic GemStone-Pharo-Bridge MagLev branch example:
//
//   session userGlobals at: #MyTestDict put: dict.
//   session commit.
//   session disconnect.
//
// Expected output includes:
//
// classic UserGlobals key: MyTestDict
// classic payload OOP: <number>
// classic loaded name: Tariq
// classic loaded amount: 100
// classic loaded currency: GBP

use gemstone_rs::{BridgeDictionary, BridgeValue, Config, Oop, Session};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;

    let payload = BridgeValue::dictionary([
        ("name".to_string(), BridgeValue::from("Tariq")),
        ("amount".to_string(), BridgeValue::from(100_i64)),
        ("currency".to_string(), BridgeValue::from("GBP")),
    ]);

    let payload_oop = payload.to_oop(&mut session)?;
    session.global_put("MyTestDict", payload_oop)?;
    session.commit()?;

    let stored = session.global_get("MyTestDict")?;
    assert_eq!(stored, payload_oop);

    let (loaded_name, loaded_amount, loaded_currency) = {
        let mut dictionary = BridgeDictionary::from_oop(&mut session, stored);
        (
            dictionary.at_string("name")?,
            dictionary.at_smallint("amount")?,
            dictionary.at_string("currency")?,
        )
    };
    assert_eq!(loaded_name, "Tariq");
    assert_eq!(loaded_amount, 100);
    assert_eq!(loaded_currency, "GBP");

    println!("classic UserGlobals key: MyTestDict");
    println!("classic payload OOP: {}", payload_oop.raw());
    println!("classic loaded name: {loaded_name}");
    println!("classic loaded amount: {loaded_amount}");
    println!("classic loaded currency: {loaded_currency}");

    // Leave the shared stone clean after the example. The Smalltalk sample
    // stops after commit; this cleanup keeps repeated Rust runs idempotent.
    session.global_put("MyTestDict", Oop::NIL)?;
    session.commit()?;
    session.logout()?;
    Ok(())
}
