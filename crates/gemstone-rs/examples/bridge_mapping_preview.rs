use gemstone_rs::{codegen, BridgeKey, BridgeValue};

fn main() {
    let payload = BridgeValue::dictionary([
        ("name".to_string(), BridgeValue::from("Tariq")),
        ("amount".to_string(), BridgeValue::from(100_i64)),
        (
            "customer".to_string(),
            BridgeValue::keyed_dictionary([
                (BridgeKey::symbol("name"), BridgeValue::from("Tariq")),
                (BridgeKey::symbol("vip"), BridgeValue::from(true)),
            ]),
        ),
        (
            "items".to_string(),
            BridgeValue::array([
                BridgeValue::dictionary([
                    ("sku".to_string(), BridgeValue::from("A-1")),
                    ("quantity".to_string(), BridgeValue::from(2_i64)),
                ]),
                BridgeValue::dictionary([
                    ("sku".to_string(), BridgeValue::from("B-2")),
                    ("quantity".to_string(), BridgeValue::from(1_i64)),
                ]),
            ]),
        ),
        (
            "state".to_string(),
            BridgeValue::Symbol("ready".to_string()),
        ),
        ("note".to_string(), BridgeValue::Nil),
    ]);

    println!(
        "{}",
        codegen::mapping_config_from_bridge_value("BookingDraft", &payload)
    );
}
