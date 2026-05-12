# gemstone-rs

`gemstone-rs` is the safe Rust client API for GemStone/S over GCI. It uses
`gemstone-gci` for runtime `libgcirpc` loading and raw ABI calls.

The Cargo package is named `gemstone-rs`; Rust code imports it as
`gemstone_rs`.

## Layout

| Crate | Purpose |
| --- | --- |
| `crates/gemstone-gci` | Low-level dynamic `libgcirpc` loader, OOP constants, and raw GCI ABI calls. |
| `crates/gemstone-rs` | Safe Rust API with `Config`, `Session`, `Oop`, `Value`, and transaction helpers. |
| `crates/gemstone-rs-cli` | CLI for evaluating Smalltalk, inspecting OOPs, browsing code, and running codegen. |
| `crates/gemstone-rs-explorer` | Local-only web explorer proving ground for browse, inspect, eval, and codegen endpoints. |
| `vscode-gemstone-rs-workbench` | Thin VS Code command layer over the Rust CLI and explorer. |

`gemstone-gci` keeps unsafe C ABI calls isolated. `gemstone-rs` is the public
crate Rust application developers should use.

## Documentation

| Guide | Link |
| --- | --- |
| Setup | [docs/setup-guide.md](docs/setup-guide.md) |
| Examples | [docs/examples-guide.md](docs/examples-guide.md) |
| User manual | [docs/user-manual.md](docs/user-manual.md) |
| Cookbook | [docs/cookbook.md](docs/cookbook.md) |
| gemstone-py vs gemstone-rs | [docs/gemstone-py-vs-gemstone-rs.md](docs/gemstone-py-vs-gemstone-rs.md) |
| Object mapping | [docs/object-mapping.md](docs/object-mapping.md) |
| Codegen | [docs/codegen.md](docs/codegen.md) |
| Explorer | [docs/explorer.md](docs/explorer.md) |
| VS Code workbench | [docs/vscode-workbench.md](docs/vscode-workbench.md) |
| Medium article | [docs/medium-article.md](docs/medium-article.md) |
| Funny introduction | [docs/funny-introduction/README.md](docs/funny-introduction/README.md) |
| PDFs | [docs/pdf/](docs/pdf/) |
| Release checklist | [docs/release-checklist.md](docs/release-checklist.md) |

## Install

For Rust applications:

```bash
cargo add gemstone-rs
```

For command-line tools:

```bash
cargo install gemstone-rs-cli
cargo install gemstone-rs-explorer
```

The CLI binary installed by `gemstone-rs-cli` is named `gemstone-rs`:

```bash
gemstone-rs --help
gemstone-rs doctor
gemstone-rs doctor --live
gemstone-rs doctor --json
gemstone-rs eval "3 + 4"
gemstone-rs-explorer --help
```

For VS Code, install the workbench from the Marketplace:

```text
https://marketplace.visualstudio.com/items?itemName=unicompute.gemstone-rs-workbench
```

GemStone environment setup:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

`GS_STONE_NAME` is also accepted as a stone-name alias. Set `GS_LIB_PATH` when
you want to point directly at a specific `libgcirpc` file.

Minimal usage:

```rust
use gemstone_rs::{Config, Session, Value};

fn main() -> gemstone_rs::Result<()> {
    let config = Config::from_env()?;
    let mut session = Session::login(config)?;

    let value = session.eval("3 + 4")?;
    assert_eq!(value, Value::SmallInt(7));

    let text = session.new_string("hello from Rust")?;
    session.global_put("GemStoneRsText", text)?;
    let stored = session.global_get("GemStoneRsText")?;
    println!("{}", session.fetch_string(stored)?);

    session.logout()?;
    Ok(())
}
```

Class-browser operations are available through the reusable library API used by
both the CLI and explorer:

```rust
use gemstone_rs::{browser::Browser, Config, Session};

let config = Config::from_env()?;
let mut session = Session::login(config)?;
let mut browser = Browser::new(&mut session);

let dictionaries = browser.dictionaries()?;
let classes = browser.classes("UserGlobals")?;
let protocols = browser.protocols("Object", false, "")?;
let methods = browser.methods("Object", "-- all --", false, "")?;
let source = browser.source("Object", "printString", false, "")?;
```

Runtime environment:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

## Examples

```bash
cargo run -p gemstone-rs --example hello_gemstone
cargo run -p gemstone-rs --example quickstart
cargo run -p gemstone-rs --example browser
cargo run -p gemstone-rs --example transactions
cargo run -p gemstone-rs --example oop_values
cargo run -p gemstone-rs --example bridge_root_mapping
cargo run -p gemstone-rs --example codegen_preview
cargo run -p gemstone-rs --example codegen_workflow
cargo run -p gemstone-rs --example generated_wrapper_app
cargo run -p gemstone-rs --example generated_mapping_app
cargo run -p gemstone-rs --example codegen_discover
```

See [examples/README.md](examples/README.md) and
[docs/examples-guide.md](docs/examples-guide.md) for the full map.

## CLI

```bash
cargo run -p gemstone-rs-cli -- doctor
cargo run -p gemstone-rs-cli -- doctor --live
cargo run -p gemstone-rs-cli -- doctor --json
cargo run -p gemstone-rs-cli -- eval "3 + 4"
cargo run -p gemstone-rs-cli -- browse dictionaries
cargo run -p gemstone-rs-cli -- browse classes UserGlobals
cargo run -p gemstone-rs-cli -- browse protocols Object
cargo run -p gemstone-rs-cli -- browse methods Object "-- all --"
cargo run -p gemstone-rs-cli -- browse source Object printString
cargo run -p gemstone-rs-cli -- inspect oop 20
cargo run -p gemstone-rs-cli -- bridge root
cargo run -p gemstone-rs-cli -- bridge keys
cargo run -p gemstone-rs-cli -- bridge get BookingDraft --symbol
cargo run -p gemstone-rs-cli -- bridge inspect BookingDraft --symbol
cargo run -p gemstone-rs-cli -- bridge put WorkbenchDraft "hello from Rust" --type String
cargo run -p gemstone-rs-cli -- bridge put WorkbenchCount 7 --type SmallInt
cargo run -p gemstone-rs-cli -- bridge remove WorkbenchDraft
cargo run -p gemstone-rs-cli -- bridge sample-config BookingDraft
cargo run -p gemstone-rs-cli -- codegen init
cargo run -p gemstone-rs-cli -- codegen preview examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen diff examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check
cargo run -p gemstone-rs-cli -- codegen generate examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen discover examples/codegen/discovered.codegen Object
```

The CLI intentionally uses only the standard library. `doctor` checks the
GemStone environment, GCI library resolution, and optionally a live `3 + 4`
probe; add `--json` when release scripts or VS Code need structured output.
`eval`, `inspect oop`, and `bridge` commands are wired to live GemStone calls.
`bridge keys` lists the keys currently stored under `GemStoneRsBridgeRoot`;
`bridge put` and `bridge remove` make explicit committed BridgeRoot edits.
The `browse` commands cover dictionaries, classes, protocols, methods, and
source using the active user's symbol list. The `codegen` commands read a
line-oriented config, preview generated Rust wrappers, diff/check stale output,
generate configs from a live stone, and write generated files.

## Codegen

Start with:

```bash
cargo run -p gemstone-rs-cli -- codegen init
```

Config example:

```text
output = src/generated/gemstone_wrappers.rs
class = Object
method = Object>>printString
method = Object>>class
```

Use `Dictionary:ClassName` when a class must be resolved from a specific
dictionary:

```text
class = UserGlobals:OkzBooking
method = UserGlobals:OkzBooking>>findById:
```

Optional metadata controls generated signatures and docs:

```text
method = UserGlobals:OkzBooking>>findById: | args=id | return=Oop | doc=Find a booking by id.
method = Object>>printString | return=String | doc=Return the receiver printString.
```

See `examples/codegen/` for a concrete generated wrapper example.

## Explorer

The explorer is intentionally local-only and read-only by default:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
```

Open:

```text
http://127.0.0.1:8787/
http://127.0.0.1:8787/api/status
http://127.0.0.1:8787/api/browse/dictionaries
http://127.0.0.1:8787/api/browse/classes?dictionary=UserGlobals
http://127.0.0.1:8787/api/browse/protocols?class=Object
http://127.0.0.1:8787/api/browse/methods?class=Object&protocol=--%20all%20--
http://127.0.0.1:8787/api/browse/source?class=Object
http://127.0.0.1:8787/api/codegen/sample
http://127.0.0.1:8787/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/check?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/bridge/root
http://127.0.0.1:8787/api/bridge/keys
http://127.0.0.1:8787/api/inspect?oop=20
```

Workspace eval is opt-in:

```bash
cargo run -p gemstone-rs-explorer -- --allow-eval
```

The first explorer pass uses standard-library HTTP only. It provides safe
defaults and API endpoints before committing to a richer UI framework.

Generate endpoints are write-gated:

```bash
cargo run -p gemstone-rs-explorer -- --allow-write
```

```text
http://127.0.0.1:8787/api/codegen/generate?config=examples/codegen/gemstone-rs.codegen
```

BridgeRoot write endpoints are also write-gated:

```bash
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchDraft&value=hello'
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchCount&value=7&value_type=SmallInt'
curl -s 'http://127.0.0.1:8787/api/bridge/remove?key=WorkbenchDraft'
```

## VS Code

`vscode-gemstone-rs-workbench` provides command palette actions that call the
Rust CLI and open output/preview editors:

```json
{
  "gemstoneRs.checkoutPath": "/path/to/gemstone-rs",
  "gemstoneRs.useCargo": true,
  "gemstoneRs.codegenConfig": "examples/codegen/gemstone-rs.codegen"
}
```

Commands include setup verification, eval, browse dictionaries/classes,
codegen init/discover/preview/diff/check/generate, and launch explorer. The
GemStone RS activity bar view browses dictionaries, classes, protocols, methods,
and the configured codegen actions. `Codegen Generate` shows the generated diff
before writing files.

## Threading

`Session` is deliberately not `Send` or `Sync`. Keep a session on the thread
that logged it in until GemStone GCI threading behavior is proven safe for
broader sharing.

## CI

The repository CI runs:

```bash
make verify
```

`make verify` runs Rust formatting, `cargo check`, clippy, tests, the codegen
freshness check for `examples/codegen/gemstone-rs.codegen`, and the VS Code
extension syntax check.

Package the VS Code extension locally with:

```bash
make vscode-package
```

That writes `vscode-gemstone-rs-workbench/gemstone-rs-workbench-0.2.4.vsix`.
The generated `.vsix` and `node_modules/` are intentionally ignored.

Verify published artifacts with:

```bash
scripts/publish_verify.sh 0.2.0
```

The verification script checks crates.io package versions, installs
`gemstone-rs-cli` and `gemstone-rs-explorer`, runs both binaries with `--help`,
and confirms the Marketplace version matches the VS Code package metadata.

## Publishing

The crates must be published in dependency order:

```bash
cargo publish -p gemstone-gci
cargo publish -p gemstone-rs
cargo publish -p gemstone-rs-cli
cargo publish -p gemstone-rs-explorer
```

Or use the repository helper:

```bash
scripts/publish_crates.sh
DRY_RUN=1 scripts/publish_crates.sh
```

Before `gemstone-gci` is published, `cargo package --workspace` is expected to
fail when it validates `gemstone-rs` against the crates.io index. Verify the
leaf crate first with:

```bash
cargo package -p gemstone-gci --no-verify
```

Run the opt-in live smoke test with:

```bash
GS_RUN_LIVE_RUST=1 cargo test -p gemstone-rs live_ -- --test-threads=1
```

The `--test-threads=1` flag avoids concurrent GCI sessions inside one Rust test
process.

## Explorer Roadmap

The first browse and codegen endpoints are now wired. Next explorer work should
add generated wrapper diffs against files and a richer frontend over the stable
local API. Embedding the explorer as a VS Code webview should remain a later
feature release.
