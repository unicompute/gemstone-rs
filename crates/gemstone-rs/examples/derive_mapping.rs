// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// derived mapped payload: BookingDraft { amount: 100, customer: CustomerDraft { name: "Tariq" }, tags: ["priority", "demo"], note: None }
// bridge root identity: <number>

use gemstone_rs::{BridgeKeyType, BridgeMapped, Config, Session};

#[derive(Clone, Debug, Eq, PartialEq, BridgeMapped)]
struct CustomerDraft {
    #[bridge(key_type = "Symbol")]
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, BridgeMapped)]
struct BookingDraft {
    #[bridge(key = "amount", key_type = "Symbol")]
    amount: i64,
    customer: CustomerDraft,
    tags: Vec<String>,
    note: Option<String>,
}

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let mut bridge_root = session.bridge_root()?;

    let draft = BookingDraft {
        amount: 100,
        customer: CustomerDraft {
            name: "Tariq".to_string(),
        },
        tags: vec!["priority".to_string(), "demo".to_string()],
        note: None,
    };

    bridge_root.transaction(|root| {
        root.put_mapped("DerivedBookingDraft", &draft)?;
        let loaded: BookingDraft = root.get_mapped("DerivedBookingDraft")?;
        assert_eq!(loaded, draft);
        root.put_with_key_type(
            "DerivedBookingDraftSymbolKey",
            BridgeKeyType::Symbol,
            "symbol-key-ok",
        )?;
        root.remove("DerivedBookingDraft")?;
        root.remove_with_key_type("DerivedBookingDraftSymbolKey", BridgeKeyType::Symbol)?;
        println!("derived mapped payload: {loaded:?}");
        println!("bridge root identity: {}", root.identity_id());
        Ok(())
    })?;

    Ok(())
}
