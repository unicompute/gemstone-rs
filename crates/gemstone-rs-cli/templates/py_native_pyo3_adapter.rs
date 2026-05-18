use gemstone_rs::py_native::{
    capabilities, compatibility_report, migration_report, samples_report, smoke_dry_run_report,
};

fn main() {
    let capabilities = capabilities();
    println!("gemstone-py-native PyO3 starter");
    println!("  contract_version: {}", capabilities.contract_version);
    println!("  operations: {}", capabilities.operations.join(", "));
    println!("  samples_json: {}", samples_report().to_json());
    println!("  smoke_json: {}", smoke_dry_run_report().to_json());
    println!("  migration_json: {}", migration_report().to_json());
    println!("  compatibility_json: {}", compatibility_report().to_json());
    println!();
    println!("Build the Python extension with:");
    println!("  python -m pip install maturin");
    println!("  maturin develop");
    println!("  python -c 'import gemstone_py_native; print(gemstone_py_native.capabilities_json())'");
    println!("  python -c 'import gemstone_py_native; print(gemstone_py_native.migration_json())'");
    println!("  python -c 'import gemstone_py_native; print(gemstone_py_native.compatibility_json())'");
}
