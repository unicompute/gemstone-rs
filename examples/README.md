# gemstone-rs Examples

Runnable examples for `gemstone-rs`.

## Start Here

Run the quickstart first. It connects to GemStone, evaluates `3 + 4`, writes a
string under `UserGlobals`, reads it back, and cleans up the temporary key.

```bash
cd /path/to/gemstone-rs
cargo run -p gemstone-rs --example quickstart
```

Most examples need the same GemStone environment:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_STONE_NAME=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

`GS_STONE_NAME` is accepted as a stone-name alias when `GS_STONE` is absent.
Set `GS_LIB_PATH` when you want to point directly at a specific `libgcirpc`
file.

## Example Map

| Role | Command | Use it when |
| --- | --- | --- |
| Hello GemStone | `cargo run -p gemstone-rs --example hello_gemstone` | You want to verify env loading, login, session id, and a tiny eval. |
| Quickstart | `cargo run -p gemstone-rs --example quickstart` | You want the smallest live read/write round trip. |
| Eval | `cargo run -p gemstone-rs --example eval` | You want only the `Session::eval("3 + 4")` shape. |
| Browser | `cargo run -p gemstone-rs --example browser` | You want dictionaries, protocols, methods, and method source. |
| Live smoke cookbook | `cargo run -p gemstone-rs --example live_smoke_cookbook` | You want login, eval, global round-trip, perform, and transaction checks in one run. |
| Transactions | `cargo run -p gemstone-rs --example transactions` | You want commit-on-success and abort-on-error behavior. |
| OOP values | `cargo run -p gemstone-rs --example oop_values` | You want explicit OOP/value conversion and export-set retention. |
| BridgeRoot mapping | `cargo run -p gemstone-rs --example bridge_root_mapping` | You want MagLev-style bridge-root storage with explicit Rust value mapping. |
| Derive mapping | `cargo run -p gemstone-rs --example derive_mapping` | You want `#[derive(BridgeMapped)]`, symbol keys, nested structs, vectors, maps, optional fields, and BridgeRoot transactions. |
| Codegen preview | `cargo run -p gemstone-rs --example codegen_preview` | You want offline wrapper generation without a live stone. |
| Codegen workflow | `cargo run -p gemstone-rs --example codegen_workflow` | You want config, preview, diff, check, and generate in one offline run. |
| Codegen discovery | `cargo run -p gemstone-rs --example codegen_discover` | You want a live-stone starter config for selected classes. |
| Mapping discovery | `cargo run -p gemstone-rs --example codegen_discover_mapping` | You want a live-stone starter mapping config for BridgeRoot payloads. |
| Generated wrapper app | `cargo run -p gemstone-rs --example generated_wrapper_app` | You want to call `Object>>printString` through checked-in generated Rust code. |
| Generated mapping app | `cargo run -p gemstone-rs --example generated_mapping_app` | You want generated `BridgeMapped` structs stored under `BridgeRoot`. |
| Codegen config | `examples/codegen/` | You want checked-in generated wrappers and CLI check/generate commands. |
| Explorer tooling | `examples/tooling/explorer.md` | You want to prove the local HTTP explorer endpoints. |
| VS Code tooling | `examples/tooling/vscode-workbench.md` | You want to prove sidebar browsing and codegen actions. |
| CLI browser walkthrough | `examples/tooling/cli-browser-walkthrough.md` | You want a terminal-only class browser workflow. |
| Axum service sketch | `examples/axum-service/README.md` | You want the recommended shape for a Rust web service. |

## Installed CLI Equivalents

After publishing or installing from crates.io:

```bash
cargo install gemstone-rs-cli
cargo install gemstone-rs-explorer
gemstone-rs doctor
gemstone-rs doctor --live
gemstone-rs doctor --json
gemstone-rs eval "3 + 4"
gemstone-rs browse dictionaries
gemstone-rs bridge root
gemstone-rs bridge keys
gemstone-rs codegen check examples/codegen/gemstone-rs.codegen
gemstone-rs-explorer --port 8787
```

## Tooling Examples

The tooling examples are command walkthroughs instead of Rust binaries:

- [Local Explorer](tooling/explorer.md)
- [VS Code Workbench](tooling/vscode-workbench.md)

Run the offline codegen workflow when you want one concrete binary example that
does config writing, preview, diff, check, and generate without needing a live
stone:

```bash
cargo run -p gemstone-rs --example codegen_workflow
```

## Expected Output

Examples that connect to GemStone/S are marked in the source comments with
`Requires a live GemStone/S stone.` These are the important success lines:

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

Offline codegen examples are marked with `Does not require a live GemStone/S
stone.` A healthy workflow run ends with:

```text
after generate: exists=true up_to_date=true
diff after generate: clean
```

## Later Examples

Good later additions, once the corresponding surfaces are stable:

- a local explorer workflow with screenshots
- a full Axum or Actix project wired into CI

## Scope

The examples intentionally stay close to the public Rust API:

- `Config::from_env`
- `Session::login`, `eval`, `execute`, `perform`, `perform_oop`
- explicit `Oop` and `Value` conversion
- `global_get` and `global_put`
- `transaction`
- `browser::Browser`
- `codegen`
- `BridgeRoot`
- `#[derive(BridgeMapped)]`

The local web explorer and VS Code extension use the same CLI/API surfaces, so
these examples are also useful when debugging higher-level tooling. For a quick
write-side tool smoke test, use `gemstone-rs bridge put-string WorkbenchDraft
"hello"` or `gemstone-rs bridge put-symbol WorkbenchState ready`, then remove
the key with `gemstone-rs bridge remove <key>`.
