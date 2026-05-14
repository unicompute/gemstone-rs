// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// generated mapped payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", tags: ["priority", "demo"], labels: {"source": "generated"}, note: Some("window seat") }

mod gemstone_wrappers {
    use gemstone_rs::{
        BridgeDictionary, BridgeFieldRead, BridgeFieldWrite, BridgeKey, BridgeKeyType,
        BridgeMapped, BridgeValue, Result,
    };
    use std::collections::BTreeMap;

    /// A typed Rust payload stored under BridgeRoot.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct BookingDraft {
        pub name: String,
        pub amount: i64,
        pub currency: String,
        pub tags: Vec<String>,
        pub labels: BTreeMap<String, String>,
        pub note: Option<String>,
    }

    impl BridgeMapped for BookingDraft {
        fn to_bridge_value(&self) -> BridgeValue {
            BridgeValue::keyed_dictionary([
                (
                    BridgeKey::new("name", BridgeKeyType::String),
                    BridgeFieldWrite::to_bridge_field_value(&self.name),
                ),
                (
                    BridgeKey::new("amount", BridgeKeyType::String),
                    BridgeFieldWrite::to_bridge_field_value(&self.amount),
                ),
                (
                    BridgeKey::new("currency", BridgeKeyType::String),
                    BridgeFieldWrite::to_bridge_field_value(&self.currency),
                ),
                (
                    BridgeKey::new("tags", BridgeKeyType::String),
                    BridgeFieldWrite::to_bridge_field_value(&self.tags),
                ),
                (
                    BridgeKey::new("labels", BridgeKeyType::String),
                    BridgeFieldWrite::to_bridge_field_value(&self.labels),
                ),
                (
                    BridgeKey::new("note", BridgeKeyType::String),
                    BridgeFieldWrite::to_bridge_field_value(&self.note),
                ),
            ])
        }

        fn from_bridge_dictionary(dictionary: &mut BridgeDictionary<'_>) -> Result<Self> {
            Ok(Self {
                name: BridgeFieldRead::read_bridge_field(
                    dictionary,
                    "name",
                    BridgeKeyType::String,
                )?,
                amount: BridgeFieldRead::read_bridge_field(
                    dictionary,
                    "amount",
                    BridgeKeyType::String,
                )?,
                currency: BridgeFieldRead::read_bridge_field(
                    dictionary,
                    "currency",
                    BridgeKeyType::String,
                )?,
                tags: BridgeFieldRead::read_bridge_field(
                    dictionary,
                    "tags",
                    BridgeKeyType::String,
                )?,
                labels: BridgeFieldRead::read_bridge_field(
                    dictionary,
                    "labels",
                    BridgeKeyType::String,
                )?,
                note: BridgeFieldRead::read_bridge_field(
                    dictionary,
                    "note",
                    BridgeKeyType::String,
                )?,
            })
        }
    }
}

use gemstone_rs::{Config, Session};
use gemstone_wrappers::BookingDraft;
use std::collections::BTreeMap;

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let mut bridge_root = session.bridge_root()?;

    let draft = BookingDraft {
        name: "Tariq".to_string(),
        amount: 100,
        currency: "GBP".to_string(),
        tags: vec!["priority".to_string(), "demo".to_string()],
        labels: BTreeMap::from([("source".to_string(), "generated".to_string())]),
        note: Some("window seat".to_string()),
    };
    bridge_root.put_mapped("GeneratedBookingDraft", &draft)?;

    let loaded: BookingDraft = bridge_root.get_mapped("GeneratedBookingDraft")?;
    assert_eq!(loaded, draft);
    println!("generated mapped payload: {loaded:?}");

    bridge_root.remove("GeneratedBookingDraft")?;
    bridge_root.commit()?;
    Ok(())
}
