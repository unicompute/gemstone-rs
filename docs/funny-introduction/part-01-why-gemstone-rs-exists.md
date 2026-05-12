# Part 1: Why gemstone-rs Exists

GemStone/S is not a SQL database wearing a new hat. It stores objects, runs
Smalltalk, and lets sessions send messages to live objects. Rust is not a
dynamic scripting language. It asks you to be clear about ownership, errors,
and resources.

That sounds like a mismatch until you notice the useful part: GemStone wants a
client that respects session boundaries, transactions, and object identity.
Rust wants APIs that make those boundaries explicit. `gemstone-rs` exists at
that meeting point.

## The First Promise

The first promise is small:

```rust
use gemstone_rs::{Config, Session, Value};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    assert_eq!(session.eval("3 + 4")?, Value::SmallInt(7));
    Ok(())
}
```

There is no web server hidden in the middle. There is no Python process acting
as a translator. The Rust process loads GemStone's GCI library and talks to the
stone directly.

## The Second Promise

The second promise is separation:

| Crate | Job |
| --- | --- |
| `gemstone-gci` | Unsafe dynamic GCI loading and raw ABI calls. |
| `gemstone-rs` | Safe Rust API. |
| `gemstone-rs-cli` | Command-line workflows. |
| `gemstone-rs-explorer` | Local HTTP browser and codegen proving ground. |

The unsafe part is boxed into the low-level crate. The public API talks in
terms of `Config`, `Session`, `Oop`, `Value`, and `Result`.

## The First Useful Command

```bash
gemstone-rs eval "3 + 4"
```

If this works, you know the process can:

- load `libgcirpc`
- read credentials
- log in
- evaluate Smalltalk
- marshal a simple result

That is a lot of proof for one tiny command.
