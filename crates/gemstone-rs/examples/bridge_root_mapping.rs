// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// bridge root: GemStoneRsBridgeRoot
// MyTestDict OOP: <number>
// loaded payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", labels: {"source": "manual"} }
// loaded status: ready
// loaded amount: 100
// loaded approved: true
// loaded tags: ["priority", "demo"]
// loaded note: Some("front desk")
// loaded labels: {"source": "manual"}
// loaded symbol labels: {"source": "manual"}

use gemstone_rs::{
    BridgeDictionary, BridgeFieldWrite, BridgeKeyType, BridgeMapped, BridgeValue, Config, Session,
};
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

    bridge_root.put_string("MyTestStatus", "ready")?;
    bridge_root.put_smallint("MyTestAmount", payload.amount)?;
    bridge_root.put_bool("MyTestApproved", true)?;
    let tags = vec!["priority".to_string(), "demo".to_string()];
    bridge_root.put_vec("MyTestTags", &tags)?;
    let note = Some("front desk".to_string());
    bridge_root.put_optional("MyTestNote", &note)?;

    bridge_root.put_map("MyTestLabels", &payload.labels)?;
    let loaded_status = bridge_root.get_string("MyTestStatus")?;
    let loaded_amount = bridge_root.get_smallint("MyTestAmount")?;
    let loaded_approved = bridge_root.get_bool("MyTestApproved")?;
    let loaded_tags: Vec<String> = bridge_root.get_vec("MyTestTags")?;
    let loaded_note: Option<String> = bridge_root.get_optional("MyTestNote")?;
    let loaded_labels: BTreeMap<String, String> = bridge_root.get_map("MyTestLabels")?;
    assert_eq!(loaded_status, "ready");
    assert_eq!(loaded_amount, payload.amount);
    assert!(loaded_approved);
    assert_eq!(loaded_tags, tags);
    assert_eq!(loaded_note, note);
    assert_eq!(loaded_labels, payload.labels);

    bridge_root.put_map_with_key_type(
        "MyTestLabelsSymbol",
        BridgeKeyType::Symbol,
        &payload.labels,
    )?;
    let loaded_symbol_labels: BTreeMap<String, String> =
        bridge_root.get_map_with_key_type("MyTestLabelsSymbol", BridgeKeyType::Symbol)?;
    assert_eq!(loaded_symbol_labels, payload.labels);

    println!("bridge root: {}", bridge_root.name());
    println!("bridge root OOP: {}", bridge_root.oop().raw());
    println!("MyTestDict OOP: {}", stored.raw());
    println!("loaded payload: {loaded:?}");
    println!("loaded status: {loaded_status}");
    println!("loaded amount: {loaded_amount}");
    println!("loaded approved: {loaded_approved}");
    println!("loaded tags: {loaded_tags:?}");
    println!("loaded note: {loaded_note:?}");
    println!("loaded labels: {loaded_labels:?}");
    println!("loaded symbol labels: {loaded_symbol_labels:?}");

    bridge_root.remove("MyTestLabels")?;
    bridge_root.remove("MyTestStatus")?;
    bridge_root.remove("MyTestAmount")?;
    bridge_root.remove("MyTestApproved")?;
    bridge_root.remove("MyTestTags")?;
    bridge_root.remove("MyTestNote")?;
    bridge_root.remove_with_key_type("MyTestLabelsSymbol", BridgeKeyType::Symbol)?;
    bridge_root.commit()?;
    Ok(())
}
