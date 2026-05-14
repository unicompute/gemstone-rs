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

The installed CLI also carries this curated map, so users do not need a source
checkout open to discover examples:

```bash
gemstone-rs hello
gemstone-rs examples list
gemstone-rs examples show quickstart
gemstone-rs examples list --json
gemstone-rs examples map
gemstone-rs compare gemstone-py --gaps
gemstone-rs examples run codegen_preview --dry-run
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
```

`gemstone-rs examples run <name>` launches the selected Cargo example from a
source checkout. `--dry-run` prints the command without compiling or connecting
to GemStone, which makes it useful for CI and release docs checks. Use
`gemstone-rs examples map` when you want the gemstone-py-style feature stream
view across crates, examples, docs, and parity status. Use `gemstone-rs
examples scaffold <name> [path]` when you installed the CLI and want a
standalone Cargo project instead of a source-checkout example. Useful aliases
include `bridge`, `mapping`, `derive`, `codegen`, `discover`, `profiles`,
`wrapper`, `framework`, `axum`, and `http`. Some scaffolds write supporting
project files as well as Rust source; `profile_codegen_workflow` includes
`gemstone-rs.codegen` and `gemstone-rs.codegen-profiles.json`.

`gemstone-rs hello` and `gemstone-rs examples hello` do not connect to
GemStone. They mirror `gemstone-examples hello` and are useful as the first CLI
sanity check after install.

| Role | Command | Use it when |
| --- | --- | --- |
| Hello CLI | `gemstone-rs hello` | You want to verify the installed CLI without GemStone credentials. |
| Scaffold quickstart | `gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart` | You want a standalone quickstart project from the installed CLI. |
| Scaffold browser | `gemstone-rs examples scaffold browser ./gemstone-rs-browser` | You want a standalone class-browser project from the installed CLI. |
| Scaffold BridgeRoot mapping | `gemstone-rs examples scaffold bridge_root_mapping ./gemstone-rs-bridge-root-mapping` | You want a standalone BridgeRoot mapping project from the installed CLI. |
| Scaffold derive mapping | `gemstone-rs examples scaffold derive_mapping ./gemstone-rs-derive-mapping` | You want a standalone derive-mapping project from the installed CLI. |
| Scaffold codegen preview | `gemstone-rs examples scaffold codegen_preview ./gemstone-rs-codegen-preview` | You want a standalone no-live codegen preview project from the installed CLI. |
| Scaffold codegen workflow | `gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow` | You want a standalone no-live codegen workflow project from the installed CLI. |
| Scaffold codegen discovery | `gemstone-rs examples scaffold codegen_discover ./gemstone-rs-codegen-discover` | You want a standalone live discovery project from the installed CLI. |
| Scaffold mapping discovery | `gemstone-rs examples scaffold codegen_discover_mapping ./gemstone-rs-codegen-discover-mapping` | You want a standalone live mapping discovery project from the installed CLI. |
| Scaffold profile codegen | `gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen` | You want a standalone profile-driven codegen project with config and profile files. |
| Scaffold generated wrapper | `gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper` | You want a standalone generated-style wrapper app from the installed CLI. |
| Scaffold generated mapping | `gemstone-rs examples scaffold generated_mapping_app ./gemstone-rs-generated-mapping` | You want a standalone generated-style mapping app from the installed CLI. |
| Scaffold HTTP service | `gemstone-rs examples scaffold http_service ./gemstone-rs-http-service` | You want a standalone HTTP health-service project from the installed CLI. |
| Scaffold Axum service | `gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service` | You want a standalone Axum health-service project from the installed CLI. |
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
| HTTP service | `cargo run -p gemstone-rs --example http_service -- --routes` | You want a real Rust HTTP service shape without framework dependencies. |
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
gemstone-rs compare gemstone-py --gaps
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
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

$ cargo run -p gemstone-rs --example http_service -- --routes
gemstone-rs HTTP service example
  GET /
  GET /health/local
  GET /health/gemstone
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
- a checked Axum or Actix framework adapter crate wired into CI

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
