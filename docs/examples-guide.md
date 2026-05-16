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

After installing the CLI, discover the same curated map without opening the
repository docs:

```bash
gemstone-rs hello
gemstone-rs examples list
gemstone-rs examples show quickstart
gemstone-rs examples list --json
gemstone-rs examples map
gemstone-rs compare gemstone-py --status
gemstone-rs compare gemstone-py --scorecard
gemstone-rs compare gemstone-py --parity
gemstone-rs compare gemstone-py --gaps
gemstone-rs compare gemstone-py --next
gemstone-rs compare gemstone-py --totals
gemstone-rs compare gemstone-py --batches
gemstone-rs compare gemstone-js --gaps
gemstone-rs compare gemstone-js --next
gemstone-rs compare gemstone-js --totals
gemstone-rs compare gemstone-js --batches
gemstone-rs compare all --status
gemstone-rs compare all --scorecard
gemstone-rs compare all --parity
gemstone-rs compare all --next
gemstone-rs compare all --totals
gemstone-rs compare all --batches
gemstone-rs examples run codegen_preview --dry-run
gemstone-rs examples run axum_service --dry-run -- --routes
gemstone-rs examples run actix_service --dry-run -- --routes
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
```

From a source checkout, `gemstone-rs examples run <name>` can launch the
selected Cargo example directly. Use `--dry-run` when you only want to verify
the command that would run. From an installed CLI, `gemstone-rs examples
scaffold <name> [path]` writes a standalone Cargo project from embedded
templates. Current scaffold templates include `quickstart`, `browser`,
`bridge_root_mapping`, `derive_mapping`, `codegen_preview`, `codegen_workflow`,
`codegen_discover`, `codegen_discover_mapping`, `profile_codegen_workflow`,
`generated_wrapper_app`, `generated_mapping_app`, `http_service`,
`session_worker_pool`, `axum_service`, and `actix_service`; aliases include `bridge`, `mapping`,
`derive`, `codegen`, `discover`, `profiles`, `wrapper`, `framework`, `axum`,
`actix`, and `http`.
Scaffolds can include supporting project files; `profile_codegen_workflow`
writes both `gemstone-rs.codegen` and `gemstone-rs.codegen-profiles.json`.
`gemstone-rs examples map` is the Rust equivalent of `gemstone-examples
plan3-map`: it groups crates, examples, docs, and gemstone-py reference points
by feature stream. The same content is maintained in [feature-map.md](feature-map.md).

`gemstone-rs hello` is the no-live equivalent of `gemstone-examples hello`.
Use it before configuring GemStone credentials when you only want to prove the
CLI binary is installed and runnable.

## Example Map

| Feature | Command or path | What it demonstrates |
| --- | --- | --- |
| Hello CLI | `gemstone-rs hello` | Verifies the installed CLI without GemStone credentials. |
| Scaffold quickstart | `gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart` | Creates a standalone Cargo quickstart project from the installed CLI. |
| Scaffold browser | `gemstone-rs examples scaffold browser ./gemstone-rs-browser` | Creates a standalone class-browser project from the installed CLI. |
| Scaffold BridgeRoot mapping | `gemstone-rs examples scaffold bridge_root_mapping ./gemstone-rs-bridge-root-mapping` | Creates a standalone BridgeRoot mapping project from the installed CLI. |
| Scaffold derive mapping | `gemstone-rs examples scaffold derive_mapping ./gemstone-rs-derive-mapping` | Creates a standalone derive-mapping project from the installed CLI. |
| Scaffold codegen preview | `gemstone-rs examples scaffold codegen_preview ./gemstone-rs-codegen-preview` | Creates a standalone no-live codegen preview project from the installed CLI. |
| Scaffold codegen workflow | `gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow` | Creates a standalone no-live codegen preview/diff/check/generate project from the installed CLI. |
| Scaffold codegen discovery | `gemstone-rs examples scaffold codegen_discover ./gemstone-rs-codegen-discover` | Creates a standalone live discovery project from the installed CLI. |
| Scaffold mapping discovery | `gemstone-rs examples scaffold codegen_discover_mapping ./gemstone-rs-codegen-discover-mapping` | Creates a standalone live mapping discovery project from the installed CLI. |
| Scaffold profile codegen | `gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen` | Creates a standalone profile-driven codegen project with config and profile files. |
| Scaffold generated wrapper | `gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper` | Creates a standalone generated-style wrapper app from the installed CLI. |
| Scaffold generated mapping | `gemstone-rs examples scaffold generated_mapping_app ./gemstone-rs-generated-mapping` | Creates a standalone generated-style BridgeMapped app from the installed CLI. |
| Scaffold HTTP service | `gemstone-rs examples scaffold http_service ./gemstone-rs-http-service` | Creates a standalone Rust HTTP health-service project from the installed CLI. |
| Scaffold worker pool | `gemstone-rs examples scaffold session_worker_pool ./gemstone-rs-worker-pool` | Creates a standalone bounded SessionWorkerPool project from the installed CLI. |
| Scaffold Axum service | `gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service` | Creates a standalone Axum health-service project from the installed CLI. |
| Scaffold Actix service | `gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service` | Creates a standalone Actix Web health-service project from the installed CLI. |
| First login | `cargo run -p gemstone-rs --example hello_gemstone` | Reads env config, logs in, prints a session id, and evaluates `3 + 4`. |
| Quickstart | `cargo run -p gemstone-rs --example quickstart` | Eval, `global_put`, `global_get`, string fetch, cleanup. |
| Eval only | `cargo run -p gemstone-rs --example eval` | Minimal `Session::eval` shape. |
| Browser API | `cargo run -p gemstone-rs --example browser` | Dictionaries, protocols, methods, and source. |
| Live smoke cookbook | `cargo run -p gemstone-rs --example live_smoke_cookbook` | Login, eval, global round-trip, perform, and transaction checks in one run. |
| Transactions | `cargo run -p gemstone-rs --example transactions` | Commit-on-success and abort-on-error transaction wrapper. |
| Session worker | `cargo run -p gemstone-rs --example session_worker` | Dedicated-thread `SessionWorker` for web services and async runtimes. |
| Session worker pool | `cargo run -p gemstone-rs --example session_worker_pool` | Bounded round-robin pool of dedicated GemStone session workers. |
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
| HTTP service | `cargo run -p gemstone-rs --example http_service -- --routes` | Standard-library web service with `/`, `/health/local`, and `/health/gemstone`. |
| Axum service | `cargo run --manifest-path examples/axum-service/Cargo.toml -- --routes` | Checked Axum route shape with `gemstone-rs-axum`, `SessionWorkerPool`, and shared `gemstone_rs::web` health responses. |
| Actix service | `cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes` | Checked Actix route shape with `gemstone-rs-actix`, `SessionWorkerPool`, and shared `gemstone_rs::web` health responses. |

The Axum and Actix services use the graceful health-pool startup path. They can
start with missing credentials, serve `/` and `/health/local`, and return a
`503` JSON error from `/health/gemstone` until the stone is configured. Verify
the route contract with:

```bash
python3 scripts/framework_route_smoke.py
```

The same smoke check asserts diagnostic and request-trace headers from the
adapters: `x-gemstone-rs-adapter`, `x-gemstone-rs-route`,
`x-gemstone-rs-request-id`, `x-gemstone-rs-request-method`, and
`x-gemstone-rs-request-path`. It also asserts the lifecycle headers
`x-gemstone-rs-request-lifecycle: received,handled` and
`x-gemstone-rs-request-duration-us`, which give tests and local proxies a
framework-neutral way to confirm the packaged handler ran. The checked services
also add `x-gemstone-rs-example-middleware: axum` or
`x-gemstone-rs-example-middleware: actix`, and the smoke script asserts the
marker so application middleware stays covered.

## Suggested Learning Order

1. `gemstone-rs hello`
2. `hello_gemstone`
3. `quickstart`
4. `browser`
5. `live_smoke_cookbook`
6. `transactions`
7. `session_worker`
8. `session_worker_pool`
9. `oop_values`
10. `bridge_root_mapping`
11. `derive_mapping`
11. `codegen_preview`
12. `codegen_workflow`
13. `generated_wrapper_app`
14. `generated_mapping_app`
15. `codegen_discover`
16. `codegen_discover_mapping`
17. `examples/codegen/`
18. `examples/tooling/cli-browser-walkthrough.md`
19. `examples/tooling/explorer.md`
20. `examples/tooling/vscode-workbench.md`
21. `http_service`
22. `examples/axum-service/`
23. `examples/actix-service/`

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

$ cargo run -p gemstone-rs --example http_service -- --routes
gemstone-rs HTTP service example
  GET /
  GET /health/local
  GET /health/gemstone
```

Offline examples should run without GemStone:

```text
$ cargo run -p gemstone-rs --example codegen_workflow
before generate: exists=false up_to_date=false diff_bytes=<number>
after generate: exists=true up_to_date=true
diff after generate: clean
```

The checked-in codegen sample also demonstrates typed arguments. A config line
such as `method = Object>>perform: | args=selector:Symbol` generates a Rust
method that accepts `selector: impl AsRef<str>` and creates the GemStone symbol
before dispatch. For application wrappers, use `SmallInt`, `String`, `Symbol`,
`Bool`, or the default `Oop` depending on how much conversion you want at the
generated boundary.

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
gemstone-rs compare gemstone-py --status
gemstone-rs compare gemstone-py --scorecard
gemstone-rs compare gemstone-py --parity
gemstone-rs compare gemstone-py --gaps
gemstone-rs compare gemstone-py --next
gemstone-rs compare gemstone-py --totals
gemstone-rs compare gemstone-py --batches
gemstone-rs compare gemstone-js --gaps
gemstone-rs compare gemstone-js --next
gemstone-rs compare gemstone-js --totals
gemstone-rs compare gemstone-js --batches
gemstone-rs compare all --status
gemstone-rs compare all --scorecard
gemstone-rs compare all --parity
gemstone-rs compare all --next
gemstone-rs compare all --totals
gemstone-rs compare all --batches
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
gemstone-rs examples run axum_service --dry-run -- --routes
gemstone-rs examples run actix_service --dry-run -- --routes
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
- live route smoke examples for `gemstone-rs-axum` and `gemstone-rs-actix`
- a richer class browser walkthrough with captured output
