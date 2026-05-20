// Requires a live GemStone/S stone.
//
// Rust equivalent of the MagLev-oriented GemStone-Pharo-Bridge example:
//
//   session bridgeRoot at: #MyTestDict put: payload.
//   session commitTransactionOrSignalConflict.
//   session disconnect.
//
// Expected output includes:
//
// maglev bridge root: GemStoneRsBridgeRoot
// maglev bridge root key: #MyTestDict
// maglev payload OOP: <number>
// maglev loaded name: Tariq
// maglev loaded amount: 100
// maglev loaded currency: GBP

use gemstone_rs::{BridgeKeyType, BridgeValue, Config, Session};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;

    let payload = BridgeValue::dictionary([
        ("name".to_string(), BridgeValue::from("Tariq")),
        ("amount".to_string(), BridgeValue::from(100_i64)),
        ("currency".to_string(), BridgeValue::from("GBP")),
    ]);

    let (root_name, root_oop, payload_oop, loaded_name, loaded_amount, loaded_currency) = {
        let mut bridge_root = session.bridge_root()?;
        let root_name = bridge_root.name().to_string();
        let root_oop = bridge_root.oop();

        let payload_oop =
            bridge_root.put_with_key_type("MyTestDict", BridgeKeyType::Symbol, payload)?;
        // Equivalent intent to commitTransactionOrSignalConflict: commit once
        // and return the conflict/error to the caller instead of hiding it.
        bridge_root.commit_with_retry(0)?;

        let (loaded_name, loaded_amount, loaded_currency) = {
            let mut dictionary =
                bridge_root.get_dictionary_with_key_type("MyTestDict", BridgeKeyType::Symbol)?;
            (
                dictionary.at_string("name")?,
                dictionary.at_smallint("amount")?,
                dictionary.at_string("currency")?,
            )
        };
        assert_eq!(loaded_name, "Tariq");
        assert_eq!(loaded_amount, 100);
        assert_eq!(loaded_currency, "GBP");

        bridge_root.remove_with_key_type("MyTestDict", BridgeKeyType::Symbol)?;
        bridge_root.commit_with_retry(0)?;

        (
            root_name,
            root_oop,
            payload_oop,
            loaded_name,
            loaded_amount,
            loaded_currency,
        )
    };

    println!("maglev bridge root: {root_name}");
    println!("maglev bridge root OOP: {}", root_oop.raw());
    println!("maglev bridge root key: #MyTestDict");
    println!("maglev payload OOP: {}", payload_oop.raw());
    println!("maglev loaded name: {loaded_name}");
    println!("maglev loaded amount: {loaded_amount}");
    println!("maglev loaded currency: {loaded_currency}");

    session.logout()?;
    Ok(())
}
