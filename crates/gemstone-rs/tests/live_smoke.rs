#[path = "fixtures/gemstone_wrappers.rs"]
mod gemstone_wrappers;

use gemstone_rs::{browser::Browser, browser::ALL_PROTOCOLS, Config, Session, Value};

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

#[test]
fn live_login_logout_when_enabled() -> gemstone_rs::Result<()> {
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
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    assert_eq!(session.eval("3 + 4")?, Value::SmallInt(7));
    Ok(())
}

#[test]
fn live_browser_resolves_object_when_enabled() -> gemstone_rs::Result<()> {
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
    let Some(config) = live_config()? else {
        return Ok(());
    };

    let mut session = Session::login(config)?;
    let oop = session.smallint_oop(7);
    let mut object = gemstone_wrappers::Object::from_oop(&mut session, oop);
    assert_eq!(object.print_string()?, "7");
    Ok(())
}
