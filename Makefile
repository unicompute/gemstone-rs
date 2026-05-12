.PHONY: verify rust-check codegen-check vscode-check vscode-package package-gci clean-vscode

verify: rust-check codegen-check vscode-check

rust-check:
	cargo fmt --all --check
	cargo check --workspace
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

codegen-check:
	cargo run -p gemstone-rs-cli -- codegen check examples/codegen/gemstone-rs.codegen

vscode-check:
	cd vscode-gemstone-rs-workbench && npm run check

vscode-package:
	cd vscode-gemstone-rs-workbench && npm ci
	cd vscode-gemstone-rs-workbench && npm run package -- --out gemstone-rs-workbench-0.1.0.vsix

package-gci:
	cargo package -p gemstone-gci --no-verify

clean-vscode:
	rm -rf vscode-gemstone-rs-workbench/node_modules vscode-gemstone-rs-workbench/*.vsix
