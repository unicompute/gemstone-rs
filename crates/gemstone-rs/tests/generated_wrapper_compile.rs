#[allow(dead_code)]
#[path = "../../../examples/codegen/generated/gemstone_wrappers.rs"]
mod gemstone_wrappers;

use gemstone_wrappers::BookingDraft;

fn assert_bridge_mapped<T: gemstone_rs::BridgeMapped>() {}

#[test]
fn checked_in_generated_wrappers_compile() {
    assert_bridge_mapped::<BookingDraft>();

    let draft = BookingDraft {
        name: "Tariq".to_string(),
        amount: 100,
        currency: "GBP".to_string(),
        tags: vec!["priority".to_string(), "demo".to_string()],
        note: None,
    };

    assert_eq!(draft.name, "Tariq");
    assert_eq!(draft.amount, 100);
    assert_eq!(draft.currency, "GBP");
    assert_eq!(draft.tags, vec!["priority".to_string(), "demo".to_string()]);
    assert_eq!(draft.note, None);
}
