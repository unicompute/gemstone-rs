# gemstone-rs-macros

Derive macros for `gemstone-rs`.

Most users should depend on `gemstone-rs` and import the re-exported
`BridgeMapped` derive from there:

```rust
use gemstone_rs::BridgeMapped;
use std::collections::BTreeMap;

#[derive(Clone, Debug, BridgeMapped)]
struct BookingDraft {
    name: String,
    amount: i64,
    labels: BTreeMap<String, String>,
    note: Option<String>,
}
```

Generic fields use the core `BridgeFieldRead` / `BridgeFieldWrite`
implementations from `gemstone-rs`. `BTreeMap<String, T>` stores a string-keyed
GemStone dictionary, and `Option<T>` writes `None` as GemStone `nil` while
missing or nil keys read back as `None`.
