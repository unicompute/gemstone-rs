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
```

The config format is intentionally line-oriented:

```text
output = generated/gemstone_wrappers.rs
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

An empty dictionary resolves through the active user's symbol list.
