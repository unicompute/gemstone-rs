// Requires a live GemStone/S stone.
//
// Expected output includes:
//
// discovered config:
// discovered classes: 1

use gemstone_rs::{codegen, Config, Session};

fn main() -> codegen::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let class_ref = codegen::ClassRef::parse("Object").expect("Object is a valid class reference");
    let output = std::env::temp_dir()
        .join(format!("gemstone-rs-discovered-{}", std::process::id()))
        .join("gemstone_wrappers.rs");

    let config = codegen::discover(&mut session, output, &[class_ref])?;
    println!("discovered config:\n{}", codegen::config_source(&config));
    println!("discovered classes: {}", config.classes.len());
    println!(
        "first class method count: {}",
        config
            .classes
            .first()
            .map_or(0, |class| class.methods.len())
    );

    Ok(())
}
