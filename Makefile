VERSION ?= 0.2.2
VSIX_VERSION ?= $(shell node -p "require('./vscode-gemstone-rs-workbench/package.json').version")

.PHONY: verify version-check rust-check codegen-check schema-check profile-check explorer-smoke vscode-check vscode-package docs-pdf docs-pdf-check release-artifact-check screenshots package-gci publish-verify release-all clean-vscode

verify: version-check rust-check codegen-check schema-check profile-check explorer-smoke vscode-check docs-pdf-check

version-check:
	python3 scripts/version_check.py

rust-check:
	cargo fmt --all --check
	cargo check --workspace
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

codegen-check:
	cargo run -p gemstone-rs-cli -- env sample
	cargo run -p gemstone-rs-cli -- env write /tmp/gemstone-rs.env --force
	cargo run -p gemstone-rs-cli -- codegen check examples/codegen/gemstone-rs.codegen
	cargo run -p gemstone-rs-cli -- --env-file /tmp/gemstone-rs.env codegen check examples/codegen/gemstone-rs.codegen
	cargo run -p gemstone-rs-cli -- codegen explain examples/codegen/gemstone-rs.codegen
	cargo run -p gemstone-rs-cli -- codegen explain --json examples/codegen/gemstone-rs.codegen
	cargo run -p gemstone-rs-cli -- codegen check-profile default examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- codegen explain-profile --json default examples/codegen/gemstone-rs.codegen-profiles.json
	cargo test --manifest-path examples/codegen-wrapper-check/Cargo.toml

schema-check:
	node scripts/validate_codegen_schemas.js

profile-check:
	cargo run -p gemstone-rs-cli -- profile validate examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile validate --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile list --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile show default --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile resolve default --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile check examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile check --json examples/codegen/gemstone-rs.codegen-profiles.json
	cargo run -p gemstone-rs-cli -- profile sample > /tmp/gemstone-rs.codegen-profiles.json
	diff -u examples/codegen/gemstone-rs.codegen-profiles.json /tmp/gemstone-rs.codegen-profiles.json
	node scripts/profile_import_summary_test.js

explorer-smoke:
	python3 scripts/explorer_endpoint_smoke.py

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

release-artifact-check:
	python3 scripts/verify_release_artifacts.py

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
