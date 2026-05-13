// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// bridge root: GemStoneRsBridgeRoot
// MyTestDict OOP: <number>
// loaded payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", labels: {"source": "manual"} }

use gemstone_rs::{BridgeDictionary, BridgeFieldWrite, BridgeMapped, BridgeValue, Config, Session};
use std::collections::BTreeMap;

#[derive(Debug, Eq, PartialEq)]
struct BookingDraft {
    name: String,
    amount: i64,
    currency: String,
    labels: BTreeMap<String, String>,
}

impl BridgeMapped for BookingDraft {
    fn to_bridge_value(&self) -> BridgeValue {
        BridgeValue::dictionary([
            ("name".to_string(), BridgeValue::from(self.name.clone())),
            ("amount".to_string(), BridgeValue::from(self.amount)),
            (
                "currency".to_string(),
                BridgeValue::from(self.currency.clone()),
            ),
            (
                "labels".to_string(),
                BridgeFieldWrite::to_bridge_field_value(&self.labels),
            ),
        ])
    }

    fn from_bridge_dictionary(dictionary: &mut BridgeDictionary<'_>) -> gemstone_rs::Result<Self> {
        Ok(Self {
            name: dictionary.at_string("name")?,
            amount: dictionary.at_smallint("amount")?,
            currency: dictionary.at_string("currency")?,
            labels: dictionary.at_map("labels")?,
        })
    }
}

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;

    let payload = BookingDraft {
        name: "Tariq".to_string(),
        amount: 100,
        currency: "GBP".to_string(),
        labels: BTreeMap::from([("source".to_string(), "manual".to_string())]),
    };

    let mut bridge_root = session.bridge_root()?;
    let payload_oop = bridge_root.put_mapped("MyTestDict", &payload)?;
    let stored = bridge_root.get_oop("MyTestDict")?;
    assert_eq!(payload_oop, stored);

    let loaded: BookingDraft = bridge_root.get_mapped("MyTestDict")?;
    assert_eq!(loaded, payload);

    println!("bridge root: {}", bridge_root.name());
    println!("bridge root OOP: {}", bridge_root.oop().raw());
    println!("MyTestDict OOP: {}", stored.raw());
    println!("loaded payload: {loaded:?}");

    bridge_root.commit()?;
    Ok(())
}
