// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// dynamic BridgeValue: Dictionary({"customer": Dictionary({"name": String("Tariq"), "vip": Bool(true)}), "items": Array([Dictionary({"quantity": SmallInt(2), "sku": String("A-1")}), Dictionary({"quantity": SmallInt(1), "sku": String("B-2")})]), "note": Nil, "state": Symbol("ready")})
// shape nodes: <number> max depth: <number>
// relationship: value.items[1].sku string
// bridge root identity: <number>
// bridge root key count: <number>

use gemstone_rs::{BridgeValue, Config, Session};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let mut bridge_root = session.bridge_root()?;

    let payload = BridgeValue::dictionary([
        (
            "customer".to_string(),
            BridgeValue::dictionary([
                ("name".to_string(), BridgeValue::from("Tariq")),
                ("vip".to_string(), BridgeValue::from(true)),
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

    bridge_root.transaction(|root| {
        root.put("BridgeValueInspection", payload.clone())?;
        let dynamic = root.get_bridge_value("BridgeValueInspection")?;
        assert_eq!(dynamic, payload);

        println!("dynamic BridgeValue: {dynamic:?}");
        let shape = dynamic.shape_report();
        println!(
            "shape nodes: {} max depth: {}",
            shape.total_nodes, shape.max_depth
        );
        for node in shape.nodes.iter().filter(|node| node.child_count == 0) {
            println!("relationship: {} {}", node.path, node.kind);
        }
        println!("bridge root identity: {}", root.identity_id());
        println!("bridge root key count: {}", root.keys()?.len());
        root.remove("BridgeValueInspection")?;
        Ok(())
    })?;

    Ok(())
}
