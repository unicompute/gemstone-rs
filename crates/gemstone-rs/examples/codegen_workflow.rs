// Does not require a live GemStone/S stone.
//
// Expected output includes:
//
// before generate: exists=false up_to_date=false
// after generate: exists=true up_to_date=true
// diff after generate: clean

use gemstone_rs::codegen::{self, Config};
use std::fs;

fn main() -> codegen::Result<()> {
    let root = std::env::temp_dir().join(format!(
        "gemstone-rs-codegen-workflow-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;

    let config_path = root.join("gemstone-rs.codegen");
    fs::write(
        &config_path,
        "# gemstone-rs codegen workflow example\n\
         output = generated/gemstone_wrappers.rs\n\
         class = Object\n\
         method = Object>>printString | return=String | doc=Return the receiver printString.\n\
         method = Object>>class\n",
    )?;

    let config = Config::from_file(&config_path)?;
    println!("config: {}", config_path.display());
    println!("output: {}", config.output.display());

    let preview = codegen::generate(&config);
    println!("preview bytes: {}", preview.source.len());
    println!(
        "preview first line: {}",
        preview.source.lines().next().unwrap_or_default()
    );

    let before = codegen::diff(&config)?;
    println!(
        "before generate: exists={} up_to_date={} diff_bytes={}",
        before.exists,
        before.up_to_date,
        before.diff.len()
    );
    assert!(!before.up_to_date);

    let generated = codegen::generate_to_file(&config)?;
    println!("generated: {}", generated.output.display());

    let check = codegen::check(&config)?;
    println!(
        "after generate: exists={} up_to_date={}",
        check.exists, check.up_to_date
    );
    assert!(check.exists);
    assert!(check.up_to_date);

    let after = codegen::diff(&config)?;
    assert!(after.diff.is_empty());
    println!("diff after generate: clean");

    Ok(())
}
