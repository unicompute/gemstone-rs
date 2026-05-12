# Release Checklist

Use this checklist for coordinated crate, VSIX, and GitHub releases.

## Before Release

- Update crate versions in `Cargo.toml` files.
- Update `vscode-gemstone-rs-workbench/package.json`.
- Regenerate codegen examples:

```bash
cargo run -p gemstone-rs-cli -- codegen generate examples/codegen/gemstone-rs.codegen
```

- Run local verification:

```bash
make verify
make vscode-package
python3 docs/build_pdf_docs.py
```

## Publish

Set GitHub secrets once:

```bash
gh secret set CARGO_REGISTRY_TOKEN
gh secret set VSCE_PAT
```

Run the release workflow:

```bash
gh workflow run release.yml \
  --ref main \
  -f version=0.2.0 \
  -f publish-crates=true \
  -f publish-vsix=true \
  -f create-github-release=true \
  -f dry-run=false
```

The workflow publishes crates in dependency order:

1. `gemstone-gci`
2. `gemstone-rs-macros`
3. `gemstone-rs`
4. `gemstone-rs-cli`
5. `gemstone-rs-explorer`

It also packages/publishes the VSIX and can create the GitHub release with the
VSIX attached.

Manual crate publishing remains available:

```bash
scripts/publish_crates.sh
```

For a dry run:

```bash
DRY_RUN=1 scripts/publish_crates.sh
```

Dry-run mode checks each crate's publish file list locally. The real publish
path still uses `cargo publish` in dependency order so crates.io resolves each
new dependency after it has been published.

## Post-Release Verification

```bash
scripts/publish_verify.sh 0.2.0
```

The script checks:

- crates.io package/version JSON for all crates
- `cargo install gemstone-rs-cli`
- `cargo install gemstone-rs-explorer`
- `gemstone-rs --help`
- `gemstone-rs-explorer --help`
- VS Code Marketplace version matches `package.json`
- GitHub Release `v<version>` exists
- GitHub Release assets include the VSIX and `SHA256SUMS`

To skip the GitHub Release asset check during early testing:

```bash
VERIFY_GITHUB_RELEASE=0 scripts/publish_verify.sh 0.2.0
```

## Optional Live Smoke

Run the manual GitHub Actions live job with GemStone secrets configured, or run
locally:

```bash
GS_RUN_LIVE_RUST=1 cargo test -p gemstone-rs live_ -- --test-threads=1
```

The live smoke coverage includes login/logout, `3 + 4 == 7`, global put/get,
string round-trip, `perform`, commit, abort, browser lookup of `Object`, and
generated wrapper `printString`.

The `--test-threads=1` flag is intentional. GemStone GCI has process-global
session state, so live tests should not be run concurrently.
