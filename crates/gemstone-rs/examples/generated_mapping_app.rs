// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// generated mapped payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", tags: ["priority", "demo"], note: Some("window seat") }

#[allow(dead_code)]
#[path = "../../../examples/codegen/generated/gemstone_wrappers.rs"]
mod gemstone_wrappers;

use gemstone_rs::{Config, Session};
use gemstone_wrappers::BookingDraft;

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let mut bridge_root = session.bridge_root()?;

    let draft = BookingDraft {
        name: "Tariq".to_string(),
        amount: 100,
        currency: "GBP".to_string(),
        tags: vec!["priority".to_string(), "demo".to_string()],
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
