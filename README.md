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
| `crates/gemstone-rs-cli` | CLI for evaluating Smalltalk, inspecting OOPs, and future codegen commands. |
| `crates/gemstone-rs-explorer` | Local-only web explorer proving ground for browse, inspect, eval, and codegen endpoints. |

`gemstone-gci` keeps unsafe C ABI calls isolated. `gemstone-rs` is the public
crate Rust application developers should use.

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

Runtime environment:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

`GS_STONE_NAME` is also accepted as a stone-name alias. Set `GS_LIB_PATH` when
you want to point directly at a specific `libgcirpc` file.

## CLI

```bash
cargo run -p gemstone-rs-cli -- eval "3 + 4"
cargo run -p gemstone-rs-cli -- browse classes
cargo run -p gemstone-rs-cli -- inspect oop 20
cargo run -p gemstone-rs-cli -- codegen check
cargo run -p gemstone-rs-cli -- codegen generate
```

The initial CLI intentionally uses only the standard library. The `eval` and
`inspect oop` commands are wired to live GemStone calls. The codegen commands
are placeholders for the next API/codegen layer.

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
http://127.0.0.1:8787/api/inspect?oop=20
```

Workspace eval is opt-in:

```bash
cargo run -p gemstone-rs-explorer -- --allow-eval
```

The first explorer pass uses standard-library HTTP only. It provides safe
defaults and API endpoints before committing to a richer UI framework.

## Threading

`Session` is deliberately not `Send` or `Sync`. Keep a session on the thread
that logged it in until GemStone GCI threading behavior is proven safe for
broader sharing.

## CI

The repository CI runs:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Publishing

The crates must be published in dependency order:

```bash
cargo publish -p gemstone-gci
cargo publish -p gemstone-rs
cargo publish -p gemstone-rs-cli
cargo publish -p gemstone-rs-explorer
```

Before `gemstone-gci` is published, `cargo package --workspace` is expected to
fail when it validates `gemstone-rs` against the crates.io index. Verify the
leaf crate first with:

```bash
cargo package -p gemstone-gci --no-verify
```

Run the opt-in live smoke test with:

```bash
GS_RUN_LIVE_RUST=1 cargo test -p gemstone-rs live_
```

## Explorer Roadmap

Next explorer work should add structured endpoints for classes, methods,
protocols, source, generated wrapper previews, and diffs against files.
