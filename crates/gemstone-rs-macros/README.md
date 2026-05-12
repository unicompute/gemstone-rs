# gemstone-rs-macros

Derive macros for `gemstone-rs`.

Most users should depend on `gemstone-rs` and import the re-exported
`BridgeMapped` derive from there:

```rust
use gemstone_rs::BridgeMapped;

#[derive(Clone, Debug, BridgeMapped)]
struct BookingDraft {
    name: String,
    amount: i64,
}
```
