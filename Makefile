VERSION ?= 0.2.2
VSIX_VERSION ?= $(shell node -p "require('./vscode-gemstone-rs-workbench/package.json').version")

.PHONY: verify version-check crate-metadata-check rust-check examples-check codegen-check schema-check profile-check release-script-check explorer-smoke vscode-check vscode-package docs-pdf docs-pdf-check release-artifact-check screenshots package-gci publish-verify release-all clean-vscode

verify: version-check crate-metadata-check rust-check examples-check codegen-check schema-check profile-check release-script-check explorer-smoke vscode-check docs-pdf-check

version-check:
	python3 scripts/version_check.py

crate-metadata-check:
	python3 scripts/crate_metadata_check.py

rust-check:
	cargo fmt --all --check
	cargo check --workspace
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

examples-check:
	cargo run -p gemstone-rs-cli -- hello
	cargo run -p gemstone-rs-cli -- hello --json
	cargo run -p gemstone-rs-cli -- compare gemstone-py
	cargo run -p gemstone-rs-cli -- compare gemstone-py --json
	cargo run -p gemstone-rs-cli -- compare gemstone-py --gaps
	cargo run -p gemstone-rs-cli -- compare gemstone-py --gaps --json
	cargo run -p gemstone-rs-cli -- examples list
	cargo run -p gemstone-rs-cli -- examples list --json
	cargo run -p gemstone-rs-cli -- examples map
	cargo run -p gemstone-rs-cli -- examples map --json
	cargo run -p gemstone-rs-cli -- examples hello
	cargo run -p gemstone-rs-cli -- examples show quickstart
	cargo run -p gemstone-rs-cli -- examples show quickstart --json
	cargo run -p gemstone-rs-cli -- examples run codegen_preview --dry-run
	cargo run -p gemstone-rs-cli -- examples run session_worker --dry-run
	cargo run -p gemstone-rs-cli -- examples run http_service --dry-run -- --routes
	cargo run -p gemstone-rs-cli -- examples run axum_service --dry-run -- --routes
	cargo run -p gemstone-rs-cli -- examples run actix_service --dry-run -- --routes
	cargo run -p gemstone-rs-cli -- examples scaffold quickstart /tmp/gemstone-rs-scaffold-quickstart --force
	cargo run -p gemstone-rs-cli -- examples scaffold browser /tmp/gemstone-rs-scaffold-browser --force
	cargo run -p gemstone-rs-cli -- examples scaffold bridge_root_mapping /tmp/gemstone-rs-scaffold-bridge-root-mapping --force
	cargo run -p gemstone-rs-cli -- examples scaffold derive_mapping /tmp/gemstone-rs-scaffold-derive-mapping --force
	cargo run -p gemstone-rs-cli -- examples scaffold codegen_preview /tmp/gemstone-rs-scaffold-codegen-preview --force
	cargo run -p gemstone-rs-cli -- examples scaffold codegen_workflow /tmp/gemstone-rs-scaffold-codegen-workflow --force
	cargo run -p gemstone-rs-cli -- examples scaffold codegen_discover /tmp/gemstone-rs-scaffold-codegen-discover --force
	cargo run -p gemstone-rs-cli -- examples scaffold codegen_discover_mapping /tmp/gemstone-rs-scaffold-codegen-discover-mapping --force
	cargo run -p gemstone-rs-cli -- examples scaffold profile_codegen_workflow /tmp/gemstone-rs-scaffold-profile-codegen-workflow --force
	cargo run -p gemstone-rs-cli -- examples scaffold generated_wrapper_app /tmp/gemstone-rs-scaffold-generated-wrapper-app --force
	cargo run -p gemstone-rs-cli -- examples scaffold generated_mapping_app /tmp/gemstone-rs-scaffold-generated-mapping-app --force
	cargo run -p gemstone-rs-cli -- examples scaffold http_service /tmp/gemstone-rs-scaffold-http-service --force
	cargo run -p gemstone-rs-cli -- examples scaffold axum_service /tmp/gemstone-rs-scaffold-axum-service --force
	cargo run -p gemstone-rs-cli -- examples scaffold actix_service /tmp/gemstone-rs-scaffold-actix-service --force
	cargo run -p gemstone-rs --example http_service -- --routes
	cargo test --manifest-path examples/axum-service/Cargo.toml
	cargo run --manifest-path examples/axum-service/Cargo.toml -- --routes
	cargo test --manifest-path examples/actix-service/Cargo.toml
	cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes

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

release-script-check:
	python3 scripts/release_asset_checks_test.py

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
