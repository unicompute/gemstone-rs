use gemstone_rs::{BridgeMapped, Config, MaterializationProfile, Remote, Session};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, BridgeMapped)]
struct BookingDraft {
    status: String,
    amount: i64,
    labels: BTreeMap<String, String>,
}

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let key = "RemoteBookingDraft";

    let initial = BookingDraft {
        status: "draft".to_string(),
        amount: 100,
        labels: BTreeMap::from([("source".to_string(), "remote-example".to_string())]),
    };

    let oop = {
        let mut bridge_root = session.bridge_root()?;
        bridge_root.put_mapped(key, &initial)?;
        bridge_root.get_oop(key)?
    };

    let mut remote = Remote::<BookingDraft>::with_type(oop, "UserGlobals:BookingDraft")
        .with_profile(MaterializationProfile::deep(4));

    let loaded = remote.refresh(&mut session)?.clone();
    assert_eq!(loaded, initial);
    println!("remote loaded: {loaded:?}");

    let mut updated = loaded;
    updated.status = "confirmed".to_string();
    remote.set_value(updated.clone());
    assert!(remote.is_dirty());
    remote.save(&mut session)?;
    assert!(!remote.is_dirty());

    let loaded_again = {
        let mut bridge_root = session.bridge_root()?;
        let loaded_again: BookingDraft = bridge_root.get_mapped(key)?;
        bridge_root.remove(key)?;
        bridge_root.commit()?;
        loaded_again
    };
    assert_eq!(loaded_again, updated);
    println!("remote saved: {loaded_again:?}");

    session.logout()?;
    Ok(())
}
