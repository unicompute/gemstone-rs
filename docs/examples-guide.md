# Examples Guide

The `gemstone-rs` examples are split between runnable Cargo examples and the
checked-in codegen sample under the repository-level `examples/` directory.

## Common Setup

```bash
gemstone-rs env sample
gemstone-rs env write
gemstone-rs doctor --env-file .env.gemstone-rs
gemstone-rs --env-file .env.gemstone-rs browse dictionaries
gemstone-rs --env-file .env.gemstone-rs codegen check examples/codegen/gemstone-rs.codegen
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
| Live smoke cookbook | `cargo run -p gemstone-rs --example live_smoke_cookbook` | Login, eval, global round-trip, perform, and transaction checks in one run. |
| Transactions | `cargo run -p gemstone-rs --example transactions` | Commit-on-success and abort-on-error transaction wrapper. |
| OOP/value conversion | `cargo run -p gemstone-rs --example oop_values` | `Value`, `Oop`, strings, symbols, and export-set retention. |
| BridgeRoot mapping | `cargo run -p gemstone-rs --example bridge_root_mapping` | MagLev-style bridge-root storage with explicit `BridgeValue` mapping. |
| Derive mapping | `cargo run -p gemstone-rs --example derive_mapping` | `#[derive(BridgeMapped)]`, symbol keys, nested structs, vectors, maps, and BridgeRoot transactions. |
| Offline codegen | `cargo run -p gemstone-rs --example codegen_preview` | Generates wrappers from the sample config without a live stone. |
| Codegen workflow | `cargo run -p gemstone-rs --example codegen_workflow` | Writes config, previews, diffs, checks, generates, and verifies a clean diff. |
| Codegen discovery | `cargo run -p gemstone-rs --example codegen_discover` | Connects to a live stone and discovers a starter config for `Object`. |
| Mapping discovery | `cargo run -p gemstone-rs --example codegen_discover_mapping` | Connects to a live stone and proposes a `BridgeMapped` config. |
| Generated wrapper app | `cargo run -p gemstone-rs --example generated_wrapper_app` | Uses checked-in generated wrappers to call `Object>>printString`. |
| Generated mapping app | `cargo run -p gemstone-rs --example generated_mapping_app` | Uses codegen-created `BridgeMapped` structs with `BridgeRoot`. |
| Generated wrapper compile check | `cargo test --manifest-path examples/codegen-wrapper-check/Cargo.toml` | Imports the checked-in generated wrappers as a separate crate. |
| Codegen files | `examples/codegen/` | Config, generated wrappers, check/diff/generate workflow. |
| Explorer tooling | `examples/tooling/explorer.md` | Local explorer startup and endpoint checks. |
| VS Code tooling | `examples/tooling/vscode-workbench.md` | Sidebar browsing, codegen actions, and explorer launch. |
| CLI browser walkthrough | `examples/tooling/cli-browser-walkthrough.md` | Terminal-only browse workflow. |
| Axum service sketch | `examples/axum-service/README.md` | Recommended web-service shape without adding workspace dependencies. |

## Suggested Learning Order

1. `hello_gemstone`
2. `quickstart`
3. `browser`
4. `live_smoke_cookbook`
5. `oop_values`
6. `transactions`
7. `bridge_root_mapping`
8. `derive_mapping`
9. `codegen_preview`
10. `codegen_workflow`
11. `generated_wrapper_app`
12. `generated_mapping_app`
13. `codegen_discover`
14. `codegen_discover_mapping`
15. `examples/codegen/`
16. `examples/tooling/cli-browser-walkthrough.md`
17. `examples/tooling/explorer.md`
18. `examples/tooling/vscode-workbench.md`
19. `examples/axum-service/README.md`

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

$ cargo run -p gemstone-rs --example live_smoke_cookbook
login ok: session <number>
eval ok: SmallInt(7)
global round-trip ok
perform ok: 7
transaction commit/abort ok

$ cargo run -p gemstone-rs --example bridge_root_mapping
bridge root: GemStoneRsBridgeRoot
MyTestDict OOP: <number>
loaded payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", labels: {"source": "manual"} }
loaded status: ready
loaded amount: 100
loaded approved: true
loaded tags: ["priority", "demo"]
loaded note: Some("front desk")
loaded labels: {"source": "manual"}
loaded symbol labels: {"source": "manual"}

$ cargo run -p gemstone-rs --example derive_mapping
derived mapped payload: BookingDraft { amount: 100, customer: CustomerDraft { name: "Tariq" }, tags: ["priority", "demo"], labels: {"source": "derive"}, note: None }

$ cargo run -p gemstone-rs --example generated_mapping_app
generated mapped payload: BookingDraft { name: "Tariq", amount: 100, currency: "GBP", tags: ["priority", "demo"], labels: {"source": "generated"}, note: Some("window seat") }
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
gemstone-rs doctor --strict
gemstone-rs doctor --json
gemstone-rs eval --env-file .env.gemstone-rs "3 + 4"
gemstone-rs browse dictionaries
gemstone-rs browse classes UserGlobals
gemstone-rs browse protocols Object
gemstone-rs browse methods Object "-- all --"
gemstone-rs browse source Object printString
gemstone-rs inspect oop 20
gemstone-rs bridge root
gemstone-rs bridge keys
gemstone-rs bridge put-string WorkbenchDraft "hello from Rust"
gemstone-rs bridge put-symbol WorkbenchState ready
gemstone-rs bridge put-smallint WorkbenchCount 7
gemstone-rs bridge put-bool WorkbenchReady true
gemstone-rs bridge remove WorkbenchDraft
gemstone-rs bridge sample-config BookingDraft
gemstone-rs codegen explain examples/codegen/gemstone-rs.codegen
gemstone-rs codegen explain --json examples/codegen/gemstone-rs.codegen
gemstone-rs codegen explain-profile --json default examples/codegen/gemstone-rs.codegen-profiles.json
cargo test --manifest-path examples/codegen-wrapper-check/Cargo.toml
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

Use the checked-in explorer profile sample when you want a repeatable browser
workflow for codegen:

```bash
cat examples/codegen/gemstone-rs.codegen-profiles.json
gemstone-rs-explorer --port 8787 --codegen-root .
```

In the explorer, click `Load Project Profiles` with
`examples/codegen/gemstone-rs.codegen-profiles.json` as the project profile
file, then select `default`, `object-wrapper`, or `bridge-mapping`.

Validate profile files before committing them:

```bash
cargo run -p gemstone-rs-cli -- profile validate examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile check examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen explain-profile --json default examples/codegen/gemstone-rs.codegen-profiles.json
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
Generate, profile-driven codegen actions, Generate Mapping Config, Preview
BridgeRoot, List BridgeRoot Keys, Put BridgeRoot String, Put BridgeRoot Symbol,
Put BridgeRoot SmallInt, Put BridgeRoot Bool, Remove BridgeRoot Key, Run
Generated Mapping Example, Load Project Profiles, Save Project Profiles, Export
Codegen Profile, Show Sample Project Profiles, Create Project Profiles,
Validate Project Profiles, List Project Profiles, Show Project Profile, Resolve
Project Profile, Check Project Profiles, and Open Docs actions.

## Later Examples

These are useful, but should wait until the corresponding APIs are stable:

- a local explorer workflow with screenshots
- a full Axum or Actix workspace member wired into CI
- a richer class browser walkthrough with captured output
