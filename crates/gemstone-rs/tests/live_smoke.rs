#[path = "fixtures/gemstone_wrappers.rs"]
mod gemstone_wrappers;

use gemstone_rs::{browser::Browser, browser::ALL_PROTOCOLS, Config, Session, Value};
use std::sync::{Mutex, MutexGuard};

static LIVE_TEST_LOCK: Mutex<()> = Mutex::new(());

fn live_enabled() -> bool {
    std::env::var("GS_RUN_LIVE_RUST").is_ok_and(|value| value == "1" || value == "true")
}

fn live_config() -> gemstone_rs::Result<Option<Config>> {
    if live_enabled() {
        Config::from_env().map(Some)
    } else {
        Ok(None)
    }
}

fn live_key(name: &str) -> String {
    format!("GemStoneRsLive{}{}", name, std::process::id())
}

fn live_test_guard() -> Option<MutexGuard<'static, ()>> {
    if live_enabled() {
        Some(
            LIVE_TEST_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    } else {
        None
    }
}

#[test]
fn live_login_logout_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    assert!(session.is_logged_in());
    session.logout()?;
    assert!(!session.is_logged_in());
    Ok(())
}

#[test]
fn live_eval_smoke_returns_seven_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    assert_eq!(session.eval("3 + 4")?, Value::SmallInt(7));
    Ok(())
}

#[test]
fn live_string_round_trip_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    let oop = session.new_string("hello from gemstone-rs live smoke")?;
    assert_eq!(
        session.fetch_string(oop)?,
        "hello from gemstone-rs live smoke"
    );
    Ok(())
}

#[test]
fn live_global_put_get_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    let key = live_key("GlobalPutGet");
    let oop = session.new_string("stored by gemstone-rs live smoke")?;
    session.global_put(&key, oop)?;
    session.commit()?;

    let stored = session.global_get(&key)?;
    assert_eq!(
        session.fetch_string(stored)?,
        "stored by gemstone-rs live smoke"
    );

    session.global_put(&key, session.nil_oop())?;
    session.commit()?;
    Ok(())
}

#[test]
fn live_perform_print_string_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    let oop = session.smallint_oop(7);
    let printed = session.perform_oop(oop, "printString", &[])?;
    assert_eq!(session.fetch_string(printed)?, "7");
    Ok(())
}

#[test]
fn live_transaction_commit_and_abort_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    let key = live_key("Transaction");

    session.transaction(|session| {
        let value = session.new_string("committed")?;
        session.global_put(&key, value)
    })?;
    let committed = session.global_get(&key)?;
    assert_eq!(session.fetch_string(committed)?, "committed");

    let aborted: gemstone_rs::Result<()> = session.transaction(|session| {
        let value = session.new_string("aborted")?;
        session.global_put(&key, value)?;
        Err(gemstone_rs::Error::IllegalOop {
            operation: "intentional live smoke abort",
        })
    });
    assert!(aborted.is_err());

    let stored = session.global_get(&key)?;
    assert_eq!(session.fetch_string(stored)?, "committed");

    session.global_put(&key, session.nil_oop())?;
    session.commit()?;
    Ok(())
}

#[test]
fn live_browser_resolves_object_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    let methods = Browser::new(&mut session).methods("Object", ALL_PROTOCOLS, false, "")?;
    assert!(methods.iter().any(|selector| selector == "printString"));
    Ok(())
}

#[test]
fn live_codegen_wrapper_print_string_when_enabled() -> gemstone_rs::Result<()> {
    let Some(_guard) = live_test_guard() else {
        return Ok(());
    };
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    let oop = session.smallint_oop(7);
    let mut object = gemstone_wrappers::Object::from_oop(&mut session, oop);
    assert_eq!(object.print_string()?, "7");
    Ok(())
}
