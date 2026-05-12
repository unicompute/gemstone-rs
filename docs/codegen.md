# Rust Codegen

`gemstone-rs` codegen creates small Rust wrapper structs around GemStone OOPs.
The goal is to move repeated selector strings into checked-in generated code
while keeping the mapping reviewable.

## Install

```bash
cargo install gemstone-rs-cli
gemstone-rs codegen --help
```

From a checkout:

```bash
cargo run -p gemstone-rs-cli -- codegen preview examples/codegen/gemstone-rs.codegen
```

## Config Format

The config is intentionally line-oriented:

```text
output = examples/codegen/generated/gemstone_wrappers.rs
class = Object
method = Object>>printString | return=String | doc=Return the receiver printString.
method = Object>>class
```

Use `Dictionary:ClassName` when a class should resolve from a specific
dictionary:

```text
class = UserGlobals:OkzBooking
method = UserGlobals:OkzBooking>>findById: | args=id | return=Oop | doc=Find a booking by id.
```

Class-side references use `class`:

```text
class = UserGlobals:OkzBooking class
method = UserGlobals:OkzBooking class>>findById: | args=id | return=Oop
```

## Method Metadata

| Option | Example | Purpose |
| --- | --- | --- |
| `args` | `args=id,user` | Names generated Rust function arguments. |
| `return` | `return=String` | Generates a typed return helper instead of `Value`. |
| `doc` | `doc=Find by id.` | Writes a Rust doc comment above the generated method. |

When `args` is omitted, codegen now infers useful argument names from selector
keywords. For example `at:put:` becomes `at, put`, and
`withCustomer:amount:` becomes `customer, amount`. Provide explicit `args=...`
when the selector keywords are not good Rust argument names.

Supported return types:

| Config value | Rust return |
| --- | --- |
| `Value` | `gemstone_rs::Value` |
| `String` | `String` |
| `SmallInt` | `i64` |
| `Bool` | `bool` |
| `Oop` | `gemstone_rs::Oop` |

## Mapped Structs

Codegen can also emit typed `BridgeMapped` structs for values stored under
`BridgeRoot`:

```text
mapped = BookingDraft | doc=A typed Rust payload stored under BridgeRoot.
field = BookingDraft.name | type=String | key=name
field = BookingDraft.amount | type=SmallInt | key=amount | key_type=Symbol
field = BookingDraft.currency | type=String | key=currency
field = BookingDraft.tags | type=Vec<String> | key=tags
```

Supported field types are `String`, `SmallInt`, `Bool`, `Oop`,
`Mapped<OtherStruct>`, and `Vec<T>`.

Field options:

| Option | Example | Purpose |
| --- | --- | --- |
| `type` | `type=Vec<String>` | Rust/GemStone field conversion. |
| `key` | `key=amount` | GemStone dictionary key. |
| `key_type` | `key_type=Symbol` | Use string keys or symbol keys explicitly. |
| `doc` | `doc=Booking amount.` | Writes a Rust doc comment above the field. |

## Commands

Create a starter config:

```bash
gemstone-rs codegen init gemstone-rs.codegen
```

Generate a config from a live stone:

```bash
gemstone-rs codegen discover gemstone-rs.codegen Object
gemstone-rs codegen discover gemstone-rs.codegen UserGlobals:OkzBooking
gemstone-rs codegen discover-mapping gemstone-rs.codegen BookingDraft Object
```

From a checkout:

```bash
cargo run -p gemstone-rs --example codegen_discover
cargo run -p gemstone-rs --example codegen_discover_mapping
```

Preview without writing:

```bash
gemstone-rs codegen preview gemstone-rs.codegen
```

Show a generated diff:

```bash
gemstone-rs codegen diff gemstone-rs.codegen
```

Check freshness in CI:

```bash
gemstone-rs codegen check gemstone-rs.codegen
```

Write generated wrappers:

```bash
gemstone-rs codegen generate gemstone-rs.codegen
```

## Generated Shape

For:

```text
method = Object>>printString | return=String | doc=Return the receiver printString.
```

The generated wrapper includes:

```rust
pub fn print_string(&mut self) -> Result<String> {
    let value = self.session.perform(self.oop, "printString", &[])?;
    match value {
        Value::String(value) => Ok(value),
        Value::Oop(oop) => self.session.fetch_string(oop),
        other => Err(Error::UnexpectedType {
            expected: "String",
            actual: format!("{other:?}"),
        }),
    }
}
```

Generated files are meant to be checked in. That makes diffs, reviews, and
editor indexing predictable.

## Generated Wrapper App

The repository includes a small live example that imports the checked-in
generated wrapper and calls `Object>>printString` on a small integer:

```bash
cargo run -p gemstone-rs --example generated_wrapper_app
```

Expected output:

```text
generated wrapper printString: 7
```

The example uses the generated `Object::from_oop` constructor:

```rust
#[path = "../../../examples/codegen/generated/gemstone_wrappers.rs"]
mod gemstone_wrappers;

use gemstone_rs::{Config, Session};

fn main() -> gemstone_rs::Result<()> {
    let mut session = Session::login(Config::from_env()?)?;
    let oop = session.smallint_oop(7);
    let mut object = gemstone_wrappers::Object::from_oop(&mut session, oop);
    println!("{}", object.print_string()?);
    Ok(())
}
```

## Generated Object Mapping

The generated file can also include Rust data structs that implement
`BridgeMapped`. Run:

```bash
cargo run -p gemstone-rs --example generated_mapping_app
```

Expected output:

```text
generated mapped payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", tags: ["priority", "demo"] }
```

The config-driven mapping emits a struct like:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookingDraft {
    pub name: String,
    pub amount: i64,
    pub currency: String,
    pub tags: Vec<String>,
}
```

and an implementation of `BridgeMapped` so it works with:

```rust
bridge_root.put_mapped("GeneratedBookingDraft", &draft)?;
let loaded: BookingDraft = bridge_root.get_mapped("GeneratedBookingDraft")?;
```

## VS Code Workflow

The `gemstone-rs Workbench` extension exposes the same flow in the GemStone RS
sidebar:

- Discover from Live Stone
- Generate Mapping Config
- Preview BridgeRoot
- Run Generated Mapping Example
- Preview Wrappers
- Diff Generated Output
- Check Freshness
- Generate Wrappers
- Open Codegen Docs

`Generate Wrappers` shows the diff first and asks before writing when output
would change.
