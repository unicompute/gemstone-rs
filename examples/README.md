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
gemstone-rs compare gemstone-py --next
gemstone-rs compare gemstone-py --batches
gemstone-rs compare all --next
gemstone-rs compare all --totals
gemstone-rs examples run codegen_preview --dry-run
gemstone-rs examples run python_native_adapter --dry-run
gemstone-rs examples run py_native_capabilities --dry-run
gemstone-rs examples run py_native_contract_fixture --dry-run
gemstone-rs examples run py_native_samples_fixture --dry-run
gemstone-rs examples run py_native_smoke_fixture --dry-run
gemstone-rs examples run py_native_migration_plan --dry-run
gemstone-rs examples run py_native_compatibility_fixture --dry-run
gemstone-rs examples run py_native_conformance_fixture --dry-run
gemstone-rs examples run py_native_handoff_bundle --dry-run
gemstone-rs examples run py_native_publish_receipt --dry-run
gemstone-rs examples run py_native_shared_core_gate --dry-run
gemstone-rs py-native capabilities --json
gemstone-rs py-native samples --json
gemstone-rs py-native migration --json
gemstone-rs py-native compatibility --json
gemstone-rs py-native conformance --json
gemstone-rs py-native handoff --json
gemstone-rs py-native publish-receipt --json
gemstone-rs py-native check-all
gemstone-rs py-native check-all --json
gemstone-rs examples run axum_service --dry-run -- --routes
gemstone-rs examples run actix_service --dry-run -- --routes
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold py_native_pyo3_adapter ./gemstone-py-native-starter
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
```

`gemstone-rs examples run <name>` launches the selected Cargo example or
CLI-backed example from a source checkout. `--dry-run` prints the command
without compiling or connecting to GemStone, which makes it useful for CI and
release docs checks. Use
`gemstone-rs examples map` when you want the gemstone-py-style feature stream
view across crates, examples, docs, and parity status. Use `gemstone-rs
examples scaffold <name> [path]` when you installed the CLI and want a
standalone Cargo project instead of a source-checkout example. Useful aliases
include `bridge`, `mapping`, `derive`, `codegen`, `discover`, `profiles`,
`wrapper`, `py-native`, `pyo3`, `framework`, `axum`, `actix`, and `http`.
Some scaffolds write supporting project files as well as Rust source;
`profile_codegen_workflow` includes `gemstone-rs.codegen` and
`gemstone-rs.codegen-profiles.json`, while `py_native_pyo3_adapter` includes a
`pyproject.toml`, `src/lib.rs`, and Python smoke tests for a thin PyO3 wrapper.
That scaffold exposes `capabilities_json`, `samples_json`,
`smoke_dry_run_json`, `migration_json`, `compatibility_json`,
`conformance_json`, and `handoff_json`, plus direct `NativeSession` methods
for eval, execute, resolve, perform, globals, export-set retention, and
transactions.
The PyO3 scaffold uses PyO3 0.28 for Python 3.14 compatibility. Source
checkouts can prove the generated starter still compiles against the local
Rust core with `python3 scripts/check_py_native_pyo3_scaffold.py`. The
`extension-module` feature is enabled by `maturin`, not by plain `cargo run`,
so the starter binary remains runnable during local smoke checks.
The py-native fixture examples now include separate value/error samples, so a
Python wrapper can test `nil`, booleans, small integers, characters, strings,
symbols, OOPs, and structured error translation without a live stone.
They also include a conformance fixture that lists the generated PyO3 module
functions, raw session methods, compatibility shim methods, fixture paths, and
scaffold files expected from a real `gemstone-py-native` wrapper.
The handoff bundle ties those fixtures together with the acceptance checks for
the downstream wrapper release: scaffold compile, fixture freshness, preserved
Python return policy, live Rust-core native smoke, and published wheel
verification from TestPyPI/PyPI.
The publish receipt fixture records the exact TestPyPI and PyPI workflow run
ids, install commands, and verified package checks for the Rust-backed
`gemstone-py-native` wheel release.
`gemstone-rs py-native check-all` is the one-command fixture gate for
downstream `gemstone-py-native` CI. It validates capabilities, samples, smoke,
compatibility, conformance, handoff, and publish-receipt fixtures together, and the
`py_native_shared_core_gate` example exposes that gate through the installed
examples catalog.

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
| Scaffold worker pool | `gemstone-rs examples scaffold session_worker_pool ./gemstone-rs-worker-pool` | You want a standalone bounded SessionWorkerPool project from the installed CLI. |
| Scaffold PyO3 adapter | `gemstone-rs examples scaffold py_native_pyo3_adapter ./gemstone-py-native-starter` | You want a starter `gemstone-py-native` crate that wraps `gemstone_rs::py_native` with PyO3 and includes a Python compatibility shim. |
| Scaffold Axum service | `gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service` | You want a standalone Axum health-service project from the installed CLI. |
| Scaffold Actix service | `gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service` | You want a standalone Actix Web health-service project from the installed CLI. |
| Hello GemStone | `cargo run -p gemstone-rs --example hello_gemstone` | You want to verify env loading, login, session id, and a tiny eval. |
| Quickstart | `cargo run -p gemstone-rs --example quickstart` | You want the smallest live read/write round trip. |
| Eval | `cargo run -p gemstone-rs --example eval` | You want only the `Session::eval("3 + 4")` shape. |
| Browser | `cargo run -p gemstone-rs --example browser` | You want dictionaries, protocols, methods, and method source. |
| Live smoke cookbook | `cargo run -p gemstone-rs --example live_smoke_cookbook` | You want login, eval, global round-trip, perform, and transaction checks in one run. |
| Transactions | `cargo run -p gemstone-rs --example transactions` | You want commit-on-success and abort-on-error behavior. |
| Session worker | `cargo run -p gemstone-rs --example session_worker` | You want a dedicated-thread worker for web services and async runtimes. |
| Session worker pool | `cargo run -p gemstone-rs --example session_worker_pool` | You want a bounded round-robin pool of dedicated GemStone session workers. |
| Async worker facade | `cargo run -p gemstone-rs --example async_worker` | You want awaitable worker-pool calls without moving `Session` across threads. |
| Python native adapter | `gemstone-rs py-native smoke --dry-run`; `cargo run -p gemstone-rs --example python_native_adapter -- --dry-run` | You want to inspect and smoke-test the Rust contract used by the `gemstone-py-native` PyO3 bridge. |
| Python native contract fixtures | `gemstone-rs py-native check examples/py-native/gemstone-rs.py-native.json`; `gemstone-rs py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json` | You want stable capability and smoke JSON samples for wrapper CI. |
| Python native value/error samples | `gemstone-rs py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json` | You want concrete value and error payload samples for Python wrapper translation tests. |
| Python native migration plan | `gemstone-rs py-native migration --json`; `gemstone-rs examples run py_native_migration_plan --dry-run` | You want the gemstone-py-native shared-core migration and publish-verification steps as a CLI/VS Code friendly report. |
| Python native compatibility fixture | `gemstone-rs py-native check-compat examples/py-native/gemstone-rs.py-native-compat.json`; `gemstone-rs examples run py_native_compatibility_fixture --dry-run` | You want the Python compatibility shim method map and return policy checked against the Rust core. |
| Python native conformance fixture | `gemstone-rs py-native check-conformance examples/py-native/gemstone-rs.py-native-conformance.json`; `gemstone-rs examples run py_native_conformance_fixture --dry-run` | You want the PyO3 module/session/shim surface checked as a downstream wrapper target. |
| Python native handoff bundle | `gemstone-rs py-native check-handoff examples/py-native/gemstone-rs.py-native-handoff.json`; `gemstone-rs examples run py_native_handoff_bundle --dry-run` | You want the downstream `gemstone-py-native` handoff artifacts and acceptance criteria checked in one place. |
| Python native publish receipt | `gemstone-rs py-native check-publish-receipt examples/py-native/gemstone-rs.py-native-publish-receipt.json`; `gemstone-rs examples run py_native_publish_receipt --dry-run` | You want the verified TestPyPI/PyPI workflow runs and install checks recorded as a fixture. |
| Python native shared-core gate | `gemstone-rs py-native check-all`; `gemstone-rs examples run py_native_shared_core_gate --dry-run` | You want all checked-in py-native fixtures validated by one downstream CI gate. |
| Python native examples runner | `gemstone-rs examples run py_native_capabilities --dry-run`; `gemstone-rs examples run py_native_contract_fixture --dry-run`; `gemstone-rs examples run py_native_samples_fixture --dry-run`; `gemstone-rs examples run py_native_smoke_fixture --dry-run`; `gemstone-rs examples run py_native_migration_plan --dry-run`; `gemstone-rs examples run py_native_compatibility_fixture --dry-run`; `gemstone-rs examples run py_native_conformance_fixture --dry-run`; `gemstone-rs examples run py_native_handoff_bundle --dry-run`; `gemstone-rs examples run py_native_publish_receipt --dry-run`; `gemstone-rs examples run py_native_shared_core_gate --dry-run` | You want the examples catalog to expose py-native fixture checks as runnable CLI-backed examples. |
| OOP values | `cargo run -p gemstone-rs --example oop_values` | You want explicit OOP/value conversion and export-set retention. |
| MagLev classic session | `cargo run -p gemstone-rs --example maglev_classic_session` | You want the Rust equivalent of the classic `session userGlobals at: #MyTestDict put: dict; commit; disconnect` example. |
| MagLev BridgeRoot session | `cargo run -p gemstone-rs --example maglev_bridge_root_session` | You want the Rust equivalent of the MagLev `session bridgeRoot at: #MyTestDict put: payload; commitTransactionOrSignalConflict` example. |
| BridgeRoot mapping | `cargo run -p gemstone-rs --example bridge_root_mapping` | You want MagLev-style bridge-root storage with explicit Rust value mapping. |
| Derive mapping | `cargo run -p gemstone-rs --example derive_mapping` | You want `#[derive(BridgeMapped)]`, symbol keys, nested structs, vectors, maps, optional fields, and BridgeRoot transactions. |
| BridgeValue inspection | `cargo run -p gemstone-rs --example bridge_value_inspection` | You want dynamic nested BridgeRoot read-back and shape reporting before committing to a typed mapping. |
| Remote object mapping | `cargo run -p gemstone-rs --example remote_object_mapping` | You want explicit `Remote<T>` refresh/save over a mapped GemStone dictionary OOP. |
| Bridge mapping preview | `cargo run -p gemstone-rs --example bridge_mapping_preview` | You want a reviewable codegen config inferred from a nested BridgeValue shape. |
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
| Axum service | `cargo run --manifest-path examples/axum-service/Cargo.toml -- --routes` | You want a checked Axum service using `gemstone-rs-axum`, `SessionWorkerPool`, and shared web health responses. |
| Actix service | `cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes` | You want a checked Actix Web service using `gemstone-rs-actix`, `SessionWorkerPool`, and shared web health responses. |

The Axum and Actix services can start before credentials are configured. They
keep `/` and `/health/local` available and return a `503` JSON error from
`/health/gemstone` until the GemStone pool is available:

```bash
python3 scripts/framework_route_smoke.py
GS_RUN_LIVE_RUST=1 python3 scripts/framework_route_smoke.py
```

The smoke check also asserts `x-gemstone-rs-adapter`,
`x-gemstone-rs-route`, `x-gemstone-rs-request-id`,
`x-gemstone-rs-request-method`, `x-gemstone-rs-request-path`,
`x-gemstone-rs-request-lifecycle`, and
`x-gemstone-rs-request-duration-us` diagnostic headers from the framework
adapters. It also checks the application middleware marker
`x-gemstone-rs-example-middleware`, `x-gemstone-rs-service`,
`x-gemstone-rs-service-version`, `cache-control: no-store`, and
`x-content-type-options: nosniff` in the checked Axum and Actix services. That
keeps application middleware and a small production-style cache/security policy
covered. Pass `--live` or set `GS_RUN_LIVE_RUST=1` when GemStone credentials
are available and `/health/gemstone` must return `{"result":7}` for both
adapters.

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
gemstone-rs compare gemstone-py --next
gemstone-rs compare gemstone-py --totals
gemstone-rs compare gemstone-py --batches
gemstone-rs compare all --next
gemstone-rs compare all --totals
gemstone-rs compare all --batches
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold py_native_pyo3_adapter ./gemstone-py-native-starter
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
gemstone-rs examples run axum_service --dry-run -- --routes
gemstone-rs examples run actix_service --dry-run -- --routes
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

$ cargo run -p gemstone-rs --example maglev_classic_session
classic UserGlobals key: MyTestDict
classic payload OOP: <number>
classic loaded name: Tariq
classic loaded amount: 100
classic loaded currency: GBP

$ cargo run -p gemstone-rs --example maglev_bridge_root_session
maglev bridge root: GemStoneRsBridgeRoot
maglev bridge root key: #MyTestDict
maglev payload OOP: <number>
maglev loaded name: Tariq
maglev loaded amount: 100
maglev loaded currency: GBP

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

$ cargo run -p gemstone-rs --example bridge_value_inspection
dynamic BridgeValue: Dictionary({"customer": Dictionary(...), "items": Array(...), "note": Nil, "state": Symbol("ready")})
bridge root identity: <number>
bridge root key count: <number>

$ cargo run -p gemstone-rs --example remote_object_mapping
remote loaded: BookingDraft { status: "draft", amount: 100, labels: {"source": "remote-example"} }
remote saved: BookingDraft { status: "confirmed", amount: 100, labels: {"source": "remote-example"} }

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

The mapping examples form a ladder:

| Start with | Then use | When |
| --- | --- | --- |
| `maglev_classic_session` | direct `UserGlobals` | You need to compare against the classic GemStone-Pharo-Bridge session example. |
| `maglev_bridge_root_session` | `BridgeRoot` plus symbol key | You need to compare against the MagLev branch `bridgeRoot at: #MyTestDict` example. |
| `bridge_value_inspection` | `BridgeValue` | You need to inspect a live dictionary/array shape before typing it. |
| `bridge_root_mapping` | `BridgeMapped` | You have stable string/symbol-keyed dictionary payloads. |
| `derive_mapping` | `#[derive(BridgeMapped)]` | You want normal Rust structs with explicit field/key annotations. |
| `remote_object_mapping` | `Remote<T>` | You need an OOP-backed cached value with explicit `refresh` and `save`. |
| `generated_mapping_app` | codegen mapping | You want generated mapping structs and wrapper tests checked into source. |
| `examples/codegen/connector-mapping.codegen` | connector metadata | You want Smalltalk selectors and return shapes recorded beside Rust fields. |

None of these examples rely on transparent persistence. GemStone reads and
writes stay at visible calls, so `&mut Session` remains the boundary for remote
work.

## Later Examples

Good later additions, once the corresponding surfaces are stable:

- a local explorer workflow with screenshots
- broader framework coverage and richer real-application middleware examples

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
