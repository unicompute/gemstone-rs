VERSION ?= 0.2.0
VSIX_VERSION ?= $(shell node -p "require('./vscode-gemstone-rs-workbench/package.json').version")

.PHONY: verify rust-check codegen-check profile-check vscode-check vscode-package docs-pdf docs-pdf-check screenshots package-gci publish-verify release-all clean-vscode

verify: rust-check codegen-check profile-check vscode-check docs-pdf-check

rust-check:
	cargo fmt --all --check
	cargo check --workspace
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

codegen-check:
	cargo run -p gemstone-rs-cli -- codegen check examples/codegen/gemstone-rs.codegen

profile-check:
	cargo run -p gemstone-rs-cli -- profile validate examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile validate --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile list --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile show default --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile sample > /tmp/gemstone-rs.codegen-profiles.json
	diff -u examples/codegen/gemstone-rs.codegen-profiles.json /tmp/gemstone-rs.codegen-profiles.json
	node scripts/profile_import_summary_test.js

vscode-check:
	cd vscode-gemstone-rs-workbench && npm run check
	cd vscode-gemstone-rs-workbench && npm run test:smoke

vscode-package:
	cd vscode-gemstone-rs-workbench && npm ci
	cd vscode-gemstone-rs-workbench && npm run package -- --out gemstone-rs-workbench-$(VSIX_VERSION).vsix

docs-pdf:
	python3 docs/build_pdf_docs.py

docs-pdf-check: docs-pdf
	test -n "$$(find docs/pdf -name '*.pdf' -type f -size +0c -print -quit)"

screenshots:
	python3 scripts/capture_explorer_screenshots.py

package-gci:
	cargo package -p gemstone-gci --no-verify

publish-verify:
	scripts/publish_verify.sh $(VERSION)

release-all:
	DRY_RUN=1 scripts/release_all.sh $(VERSION)

clean-vscode:
	rm -rf vscode-gemstone-rs-workbench/node_modules vscode-gemstone-rs-workbench/*.vsix
