VSIX_VERSION ?= $(shell node -p "require('./vscode-gemstone-rs-workbench/package.json').version")

.PHONY: verify rust-check codegen-check vscode-check vscode-package docs-pdf docs-pdf-check package-gci publish-verify clean-vscode

verify: rust-check codegen-check vscode-check docs-pdf-check

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
	cd vscode-gemstone-rs-workbench && npm run package -- --out gemstone-rs-workbench-$(VSIX_VERSION).vsix

docs-pdf:
	python3 docs/build_pdf_docs.py

docs-pdf-check: docs-pdf
	git diff --exit-code -- docs/pdf

package-gci:
	cargo package -p gemstone-gci --no-verify

publish-verify:
	scripts/publish_verify.sh 0.2.0

clean-vscode:
	rm -rf vscode-gemstone-rs-workbench/node_modules vscode-gemstone-rs-workbench/*.vsix
