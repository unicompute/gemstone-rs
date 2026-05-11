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

Run the opt-in live smoke test with:

```bash
GS_RUN_LIVE_RUST=1 cargo test -p gemstone-rs live_
```

## Explorer Roadmap

A web explorer should live as a separate crate, for example
`crates/gemstone-rs-explorer`, after the CLI and codegen APIs are stable. It
should bind to `127.0.0.1` by default, start read-only, require explicit
credentials, and make eval/write operations opt-in.
