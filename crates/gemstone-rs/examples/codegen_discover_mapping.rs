// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// discovered mapping config:
// mapped = BookingDraft

use gemstone_rs::{codegen, Config, Session};

fn main() -> codegen::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let class_ref = codegen::ClassRef::parse("Object").expect("Object is a valid class reference");
    let output = std::env::temp_dir()
        .join(format!(
            "gemstone-rs-discovered-mapping-{}",
            std::process::id()
        ))
        .join("gemstone_wrappers.rs");

    let config = codegen::discover_mapping(&mut session, output, "BookingDraft", &class_ref)?;
    println!(
        "discovered mapping config:\n{}",
        codegen::config_source(&config)
    );
    Ok(())
}
