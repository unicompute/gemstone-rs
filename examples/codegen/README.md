# gemstone-rs codegen example

This example shows the first Rust wrapper-generation workflow.

Generate wrappers:

```bash
cargo run -p gemstone-rs-cli -- codegen generate examples/codegen/gemstone-rs.codegen
```

Check whether the generated file is current:

```bash
cargo run -p gemstone-rs-cli -- codegen check examples/codegen/gemstone-rs.codegen
```

Preview without writing:

```bash
cargo run -p gemstone-rs-cli -- codegen preview examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen explain examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen explain --json examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen explain-profile --json default examples/codegen/gemstone-rs.codegen-profiles.json
cargo test --manifest-path examples/codegen-wrapper-check/Cargo.toml
```

Generated wrappers include `#[cfg(test)]` stubs for surface names, method
metadata, and mapped-field metadata. `codegen explain` summarizes the output
path, those test stubs, wrapper methods, return helpers, and mapped fields
before you write files. Add `--json` when a tool such as the explorer or VS
Code needs a structured summary.
The wrapper-check crate imports the generated file and keeps the checked-in
generated Rust compileable.

The config format is intentionally line-oriented:

```text
output = generated/gemstone_wrappers.rs
class = Object
method = Object>>printString | return=String | doc=Return the receiver printString.
method = Object>>class
method = Object>>perform: | args=selector:Symbol | doc=Perform a unary selector supplied as a Rust string.
method = Object>>_alias | return=Symbol | doc=Return the receiver alias symbol as a Rust String.
```

Use `Dictionary:ClassName` when a class must be resolved from a specific
dictionary:

```text
class = UserGlobals:OkzBooking
method = UserGlobals:OkzBooking>>findById:
```

An empty dictionary resolves through the active user's symbol list.

Optional method metadata controls generated signatures:

```text
method = UserGlobals:OkzBooking>>findById: | args=id | return=Oop | doc=Find a booking by id.
method = UserGlobals:OkzBooking class>>findById: | args=id:SmallInt | return=Oop
method = UserGlobals:User>>named:active: | args=name:String,active:Bool | return=Oop
```

Untyped `args` stay as `Oop` parameters. Typed arguments generate native Rust
parameters and convert them before `perform`: `SmallInt` becomes `i64`,
`String` and `Symbol` become `impl AsRef<str>`, and `Bool` becomes `bool`.
Typed returns can use `Value`, `Oop`, `String`, `Symbol`, `SmallInt`, or
`Bool`; `return=Symbol` fetches the symbol bytes back as a Rust `String`.

The same config can generate typed `BridgeMapped` structs:

```text
mapped = BookingDraft | doc=A typed Rust payload stored under BridgeRoot.
field = BookingDraft.name | type=String | key=name
field = BookingDraft.amount | type=SmallInt | key=amount | key_type=Symbol
field = BookingDraft.currency | type=String | key=currency
field = BookingDraft.tags | type=Vec<String> | key=tags
field = BookingDraft.labels | type=BTreeMap<String, String> | key=labels
field = BookingDraft.note | type=Option<String> | key=note
```

`BTreeMap<String, T>` fields store string-keyed relationship metadata as
GemStone dictionaries and read back through the same typed field conversion
used by nested structs and vectors. `Map<String, T>` is accepted as a shorter
alias, and `Dictionary<T>` means `BTreeMap<String, T>`.

Mapped fields support string keys, symbol keys, nested mapped structs, and
vectors. Optional fields use `Option<T>`; missing keys and GemStone `nil` read
back as `None`, while `Some(value)` reads through the inner field type.

Connector-style mapping metadata is shown in:

```text
examples/codegen/connector-mapping.codegen
```

It records the Smalltalk selector and remote return shape next to the Rust
mapped field:

```text
mapped = Booking
class = UserGlobals:OkzBooking
field = Booking.status | selector=status | return=Symbol | key=status
field = Booking.customer | selector=customer | return=Mapped<Customer> | key=customer
```

Use it when you want explorer or VS Code tooling to show how a Rust field maps
to a live GemStone selector while keeping generated persistence explicit.
Check it with:

```bash
cargo run -p gemstone-rs-cli -- codegen explain examples/codegen/connector-mapping.codegen
cargo run -p gemstone-rs-cli -- codegen explain --json examples/codegen/connector-mapping.codegen
```

Generate a config from a live stone:

```bash
cargo run -p gemstone-rs-cli -- codegen discover examples/codegen/discovered.codegen Object
cargo run -p gemstone-rs-cli -- codegen discover-mapping examples/codegen/mapping.codegen BookingDraft Object
```

Live discovery writes selectors, source-header argument names, and
protocol/source documentation into the starter config. If source is not
available, it falls back to keyword-selector argument names. It keeps
discovered argument and return types conservative as `Oop`/`Value`; edit the
generated config with `args=name:SmallInt`, `args=name:String`,
`return=String`, or other typed metadata after reviewing the live method
source.

Preview a diff before writing:

```bash
cargo run -p gemstone-rs-cli -- codegen diff examples/codegen/gemstone-rs.codegen
```

Run the checked-in generated wrapper against a live stone:

```bash
cargo run -p gemstone-rs --example generated_wrapper_app
```

Expected:

```text
generated wrapper printString: 7
```

Run live config discovery:

```bash
cargo run -p gemstone-rs --example codegen_discover
cargo run -p gemstone-rs --example codegen_discover_mapping
```

Create installed standalone codegen projects:

```bash
gemstone-rs examples scaffold codegen_discover ./gemstone-rs-codegen-discover
gemstone-rs examples scaffold codegen_discover_mapping ./gemstone-rs-codegen-discover-mapping
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
```

Run the generated mapping example:

```bash
cargo run -p gemstone-rs --example generated_mapping_app
```

Project profile sample:

```text
examples/codegen/gemstone-rs.codegen-profiles.json
```

Schema and docs:

```text
schemas/gemstone-rs.codegen-profiles.schema.json
schemas/gemstone-rs.codegen.schema.json
schemas/gemstone-rs.codegen-explain.schema.json
docs/profile-schema.md
```

Open it from the explorer Codegen Workflow panel with `Load Project Profiles`.
It includes `default`, `object-wrapper`, and `bridge-mapping` profiles for
repeatable wrapper and BridgeRoot mapping demos. `Save Project Profiles`
requires `gemstone-rs-explorer --allow-write`; server-side validation rejects
unknown fields, missing or duplicate names, invalid versions, non-string
profile fields, and `..` path traversal before writing.

Validate before committing:

```bash
cargo run -p gemstone-rs-cli -- profile sample
cargo run -p gemstone-rs-cli -- profile init gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile list examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile show default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile resolve default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile check examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile check --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen preview-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen diff-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen check-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen explain-profile --json default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen generate-profile default examples/codegen/gemstone-rs.codegen-profiles.json
```
