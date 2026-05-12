# Part 3: OOPs, Values, and the Browser

GemStone object identity is represented by OOPs. Rust programmers are already
comfortable with explicit handles, so `gemstone-rs` does not hide them.

```rust
let seven = session.smallint_oop(7);
let printed = session.perform_oop(seven, "printString", &[])?;
println!("{}", session.fetch_string(printed)?);
```

If you want a marshalled result instead, use `perform`:

```rust
let value = session.perform(seven, "class", &[])?;
println!("{value:?}");
```

## Values

Simple results become `Value`:

```rust
use gemstone_rs::Value;

match session.eval("3 + 4")? {
    Value::SmallInt(value) => println!("{value}"),
    other => println!("{other:?}"),
}
```

Everything else can stay as an `Oop`. That is useful because not every
GemStone object should pretend to be a Rust value.

## Retaining OOPs

When a raw OOP must live across calls, retain it:

```rust
let text = session.new_string("retained")?;
let handle = session.retain_oop(text)?;
println!("{}", handle.oop().raw());
```

The handle releases on drop. Your future self will appreciate that.

## Browser API

The browser API lets Rust inspect code in the stone:

```rust
use gemstone_rs::browser::{Browser, ALL_PROTOCOLS};

let mut browser = Browser::new(&mut session);
let dictionaries = browser.dictionaries()?;
let methods = browser.methods("Object", ALL_PROTOCOLS, false, "")?;
let source = browser.source("Object", "printString", false, "")?;
```

The same browser API powers:

- `gemstone-rs browse ...`
- `gemstone-rs-explorer`
- the VS Code sidebar

That is the pattern for the project: one real Rust API, multiple tools on top.
