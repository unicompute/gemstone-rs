# gemstone-rs

`gemstone-rs` is the safe Rust client API for GemStone/S over GCI. It uses
`gemstone-gci` for runtime `libgcirpc` loading and raw ABI calls.

The Cargo package is named `gemstone-rs`; Rust code imports it as
`gemstone_rs`.

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

Builder usage:

```rust
use gemstone_rs::{Config, Session};

let config = Config::builder()
    .stone("gs64stone")
    .username("DataCurator")
    .password("swordfish")
    .build()?;

let mut session = Session::login(config)?;
session.transaction(|session| {
    let value = session.new_string("hello from a transaction")?;
    session.global_put("GemStoneRsExample", value)
})?;
```

Browse classes and source through the reusable browser API:

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

`GS_STONE_NAME` is also accepted as a stone-name alias. Set `GS_LIB_PATH` when
you want to point directly at a specific `libgcirpc` file.

`Session` is deliberately not `Send` or `Sync`. Keep one session on one thread
until GemStone GCI threading behavior is proven safe for broader sharing.

Generate initial Rust wrappers with the CLI:

```bash
cargo run -p gemstone-rs-cli -- codegen init
cargo run -p gemstone-rs-cli -- codegen preview gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen generate gemstone-rs.codegen
```

Run the opt-in live smoke test with:

```bash
GS_RUN_LIVE_RUST=1 cargo test -p gemstone-rs live_eval_smoke_returns_seven_when_enabled
```
