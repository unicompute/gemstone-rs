# Examples Guide

The `gemstone-rs` examples are split between runnable Cargo examples and the
checked-in codegen sample under the repository-level `examples/` directory.

## Common Setup

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_STONE_NAME=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

Run Cargo examples from the repository root:

```bash
cd /path/to/gemstone-rs
cargo run -p gemstone-rs --example quickstart
```

## Example Map

| Feature | Command or path | What it demonstrates |
| --- | --- | --- |
| First login | `cargo run -p gemstone-rs --example hello_gemstone` | Reads env config, logs in, prints a session id, and evaluates `3 + 4`. |
| Quickstart | `cargo run -p gemstone-rs --example quickstart` | Eval, `global_put`, `global_get`, string fetch, cleanup. |
| Eval only | `cargo run -p gemstone-rs --example eval` | Minimal `Session::eval` shape. |
| Browser API | `cargo run -p gemstone-rs --example browser` | Dictionaries, protocols, methods, and source. |
| Transactions | `cargo run -p gemstone-rs --example transactions` | Commit-on-success and abort-on-error transaction wrapper. |
| OOP/value conversion | `cargo run -p gemstone-rs --example oop_values` | `Value`, `Oop`, strings, symbols, and export-set retention. |
| BridgeRoot mapping | `cargo run -p gemstone-rs --example bridge_root_mapping` | MagLev-style bridge-root storage with explicit `BridgeValue` mapping. |
| Derive mapping | `cargo run -p gemstone-rs --example derive_mapping` | `#[derive(BridgeMapped)]`, symbol keys, nested structs, vectors, and BridgeRoot transactions. |
| Offline codegen | `cargo run -p gemstone-rs --example codegen_preview` | Generates wrappers from the sample config without a live stone. |
| Codegen workflow | `cargo run -p gemstone-rs --example codegen_workflow` | Writes config, previews, diffs, checks, generates, and verifies a clean diff. |
| Codegen discovery | `cargo run -p gemstone-rs --example codegen_discover` | Connects to a live stone and discovers a starter config for `Object`. |
| Mapping discovery | `cargo run -p gemstone-rs --example codegen_discover_mapping` | Connects to a live stone and proposes a `BridgeMapped` config. |
| Generated wrapper app | `cargo run -p gemstone-rs --example generated_wrapper_app` | Uses checked-in generated wrappers to call `Object>>printString`. |
| Generated mapping app | `cargo run -p gemstone-rs --example generated_mapping_app` | Uses codegen-created `BridgeMapped` structs with `BridgeRoot`. |
| Codegen files | `examples/codegen/` | Config, generated wrappers, check/diff/generate workflow. |
| Explorer tooling | `examples/tooling/explorer.md` | Local explorer startup and endpoint checks. |
| VS Code tooling | `examples/tooling/vscode-workbench.md` | Sidebar browsing, codegen actions, and explorer launch. |

## Suggested Learning Order

1. `hello_gemstone`
2. `quickstart`
3. `browser`
4. `oop_values`
5. `transactions`
6. `bridge_root_mapping`
7. `derive_mapping`
8. `codegen_preview`
9. `codegen_workflow`
10. `generated_wrapper_app`
11. `generated_mapping_app`
12. `codegen_discover`
13. `codegen_discover_mapping`
14. `examples/codegen/`
15. `examples/tooling/explorer.md`
16. `examples/tooling/vscode-workbench.md`

## Expected Output

Live examples need GemStone credentials and a reachable stone. If the
environment is configured correctly, these are the important lines to look for:

```text
$ cargo run -p gemstone-rs --example hello_gemstone
session id: <number>
3 + 4 => SmallInt(7)

$ cargo run -p gemstone-rs --example quickstart
GemStone eval ok: SmallInt(7)
GemStoneRsQuickstart: hello from gemstone-rs quickstart

$ cargo run -p gemstone-rs --example generated_wrapper_app
generated wrapper printString: 7

$ cargo run -p gemstone-rs --example bridge_root_mapping
bridge root: GemStoneRsBridgeRoot
MyTestDict OOP: <number>
loaded payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP" }

$ cargo run -p gemstone-rs --example derive_mapping
derived mapped payload: BookingDraft { amount: 100, customer: CustomerDraft { name: "Tariq" }, tags: ["priority", "demo"] }

$ cargo run -p gemstone-rs --example generated_mapping_app
generated mapped payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", tags: ["priority", "demo"] }
```

Offline examples should run without GemStone:

```text
$ cargo run -p gemstone-rs --example codegen_workflow
before generate: exists=false up_to_date=false diff_bytes=<number>
after generate: exists=true up_to_date=true
diff after generate: clean
```

## CLI Equivalents

The CLI gives you the same live checks without compiling examples:

```bash
cargo install gemstone-rs-cli
gemstone-rs doctor
gemstone-rs doctor --live
gemstone-rs doctor --json
gemstone-rs eval "3 + 4"
gemstone-rs browse dictionaries
gemstone-rs browse classes UserGlobals
gemstone-rs browse protocols Object
gemstone-rs browse methods Object "-- all --"
gemstone-rs browse source Object printString
gemstone-rs inspect oop 20
gemstone-rs bridge root
gemstone-rs bridge keys
gemstone-rs bridge sample-config BookingDraft
```

## Codegen Workflow

```bash
cargo run -p gemstone-rs-cli -- codegen init examples/codegen/demo.codegen
cargo run -p gemstone-rs-cli -- codegen preview examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen diff examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen generate examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen discover-mapping examples/codegen/mapping.codegen BookingDraft Object
```

Generate a starter config from a live stone:

```bash
cargo run -p gemstone-rs-cli -- codegen discover examples/codegen/discovered.codegen Object
cargo run -p gemstone-rs --example codegen_discover_mapping
```
Run the generated wrapper example after checking the generated file into the
repository:

```bash
cargo run -p gemstone-rs --example generated_wrapper_app
```

## VS Code

Install the workbench from:

```text
https://marketplace.visualstudio.com/items?itemName=unicompute.gemstone-rs-workbench
```

Use the GemStone RS sidebar to browse dictionaries, classes, protocols, and
methods. The same sidebar exposes Codegen Discover, Preview, Diff, Check,
Generate, Generate Mapping Config, Preview BridgeRoot, List BridgeRoot Keys,
Put BridgeRoot String, Remove BridgeRoot Key, Run Generated Mapping Example,
and Open Docs actions.

## Later Examples

These are useful, but should wait until the corresponding APIs are stable:

- a tiny Axum or Actix web service using `gemstone-rs`
- a read-only class browser CLI walkthrough with captured output
- a local explorer workflow with screenshots
- a live smoke cookbook for CI secrets
