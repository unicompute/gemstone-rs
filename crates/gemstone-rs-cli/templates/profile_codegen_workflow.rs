// Does not require a live GemStone/S stone.
//
// This scaffold includes:
//
// - gemstone-rs.codegen
// - gemstone-rs.codegen-profiles.json
//
// Expected output includes:
//
// selected profile: default
// generated: generated/gemstone_wrappers.rs
// profile check: 3 ok, 0 stale, 0 errors

use gemstone_rs::{codegen, profiles};
use std::{
    error::Error,
    io::{Error as IoError, ErrorKind},
    path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
    let profile_path = Path::new("gemstone-rs.codegen-profiles.json");
    let project = profiles::load_file(profile_path)?;
    let profile = project.get("default").ok_or_else(|| {
        IoError::new(
            ErrorKind::NotFound,
            "default profile not found in gemstone-rs.codegen-profiles.json",
        )
    })?;
    let config_path = profile.resolved_config_path()?;

    println!("selected profile: {}", profile.name);
    println!("config: {}", config_path.display());

    let config = codegen::Config::from_file(&config_path)?;
    println!("{}", codegen::explain(&config));

    let before = codegen::check(&config)?;
    println!(
        "before generate: exists={} up_to_date={}",
        before.exists, before.up_to_date
    );

    let generated = codegen::generate_to_file(&config)?;
    println!("generated: {}", generated.output.display());

    let report = profiles::check_file(profile_path, None)?;
    let counts = report.counts();
    println!(
        "profile check: {} ok, {} stale, {} errors",
        counts.ok_count, counts.stale_count, counts.error_count
    );
    assert!(report.ok());

    Ok(())
}
