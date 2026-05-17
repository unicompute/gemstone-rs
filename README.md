# gemstone-rs

`gemstone-rs` is the safe Rust client API for GemStone/S over GCI. It uses
`gemstone-gci` for runtime `libgcirpc` loading and raw ABI calls.

The Cargo package is named `gemstone-rs`; Rust code imports it as
`gemstone_rs`.

## Layout

| Crate | Purpose |
| --- | --- |
| `crates/gemstone-gci` | Low-level dynamic `libgcirpc` loader, OOP constants, and raw GCI ABI calls. |
| `crates/gemstone-rs` | Safe Rust API with `Config`, `Session`, `Oop`, `Value`, and transaction helpers. |
| `crates/gemstone-rs-axum` | Axum route helpers for `/`, `/health/local`, and `/health/gemstone`. |
| `crates/gemstone-rs-actix` | Actix Web route helpers for `/`, `/health/local`, and `/health/gemstone`. |
| `crates/gemstone-rs-cli` | CLI for evaluating Smalltalk, inspecting OOPs, browsing code, and running codegen. |
| `crates/gemstone-rs-explorer` | Local-only web explorer proving ground for browse, inspect, eval, and codegen endpoints. |
| `vscode-gemstone-rs-workbench` | Thin VS Code command layer over the Rust CLI and explorer. |

`gemstone-gci` keeps unsafe C ABI calls isolated. `gemstone-rs` is the public
crate Rust application developers should use.

## Documentation

| Guide | Link |
| --- | --- |
| Setup | [docs/setup-guide.md](docs/setup-guide.md) |
| Examples | [docs/examples-guide.md](docs/examples-guide.md) |
| User manual | [docs/user-manual.md](docs/user-manual.md) |
| Cookbook | [docs/cookbook.md](docs/cookbook.md) |
| gemstone-py vs gemstone-rs | [docs/gemstone-py-vs-gemstone-rs.md](docs/gemstone-py-vs-gemstone-rs.md) |
| Object mapping | [docs/object-mapping.md](docs/object-mapping.md) |
| Codegen | [docs/codegen.md](docs/codegen.md) |
| Explorer | [docs/explorer.md](docs/explorer.md) |
| VS Code workbench | [docs/vscode-workbench.md](docs/vscode-workbench.md) |
| Screenshot workflow | [docs/screenshots.md](docs/screenshots.md) |
| Performance and safety | [docs/performance-safety.md](docs/performance-safety.md) |
| Shared core integration | [docs/shared-core-integration.md](docs/shared-core-integration.md) |
| Medium article | [docs/medium-article.md](docs/medium-article.md) |
| Funny introduction | [docs/funny-introduction/README.md](docs/funny-introduction/README.md) |
| PDFs | [docs/pdf/](docs/pdf/) |
| Release checklist | [docs/release-checklist.md](docs/release-checklist.md) |

## Install

For Rust applications:

```bash
cargo add gemstone-rs
cargo add gemstone-rs-axum
cargo add gemstone-rs-actix
```

For command-line tools:

```bash
cargo install gemstone-rs-cli
cargo install gemstone-rs-explorer
```

The CLI binary installed by `gemstone-rs-cli` is named `gemstone-rs`:

```bash
gemstone-rs --help
gemstone-rs hello
gemstone-rs compare gemstone-py
gemstone-rs compare gemstone-py --status
gemstone-rs compare gemstone-py --scorecard
gemstone-rs compare gemstone-py --parity
gemstone-rs compare gemstone-py --gaps
gemstone-rs compare gemstone-py --next
gemstone-rs compare gemstone-py --totals
gemstone-rs compare gemstone-py --batches
gemstone-rs compare all --status
gemstone-rs compare all --scorecard
gemstone-rs compare all --parity
gemstone-rs compare all --next
gemstone-rs compare all --totals
gemstone-rs compare all --batches
gemstone-rs py-native capabilities
gemstone-rs py-native capabilities --json
gemstone-rs py-native smoke --dry-run
gemstone-rs py-native smoke --dry-run --json
gemstone-rs env sample
gemstone-rs env write
gemstone-rs examples list
gemstone-rs examples map
gemstone-rs examples show quickstart
gemstone-rs examples run codegen_preview --dry-run
gemstone-rs examples run axum_service --dry-run -- --routes
gemstone-rs examples run actix_service --dry-run -- --routes
gemstone-rs examples run python_native_adapter --dry-run
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
gemstone-rs doctor
gemstone-rs doctor --live --strict
gemstone-rs doctor --env-file .env.gemstone-rs --live
gemstone-rs doctor --json
gemstone-rs eval --env-file .env.gemstone-rs "3 + 4"
gemstone-rs --env-file .env.gemstone-rs browse dictionaries
gemstone-rs --env-file .env.gemstone-rs codegen check gemstone-rs.codegen
gemstone-rs-explorer --env-file .env.gemstone-rs --port 8787
gemstone-rs-explorer --help
```

The py-native JSON fixture is checked in at
`examples/py-native/gemstone-rs.py-native.json`, and the dry-run smoke fixture
is checked in at `examples/py-native/gemstone-rs.py-native-smoke.json`, for
downstream wrapper CI.
Validate it with:

```bash
gemstone-rs py-native check examples/py-native/gemstone-rs.py-native.json
gemstone-rs py-native smoke --dry-run
```

`gemstone-rs compare all --totals` prints only the active gemstone-rs estimate:
**1 batch**, roughly **6-10 hours** total. Use
`gemstone-rs compare gemstone-py --status` for the shortest answer with
parity score and batch count, `gemstone-rs compare gemstone-py --scorecard`
for the decision view, `gemstone-rs compare gemstone-py --parity` for
area-by-area maturity scores, or `gemstone-rs compare all --batches` for the
per-batch detail and verification commands.

For VS Code, install the workbench from the Marketplace:

```text
https://marketplace.visualstudio.com/items?itemName=unicompute.gemstone-rs-workbench
```

GemStone environment setup:

```bash
gemstone-rs env sample
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

`GS_STONE_NAME` is also accepted as a stone-name alias. Set `GS_LIB_PATH` when
you want to point directly at a specific `libgcirpc` file.

Minimal usage:

```rust
use gemstone_rs::{Config, Session, Value};

fn main() -> gemstone_rs::Result<()> {
    let config = Config::from_env()?;
    let mut session = Session::login(config)?;

    let value = session.eval("3 + 4")?;
    assert_eq!(value, Value::SmallInt(7));

    let text = session.new_string("hello from Rust")?;
    session.global_put("GemStoneRsText", text)?;
    let stored = session.global_get("GemStoneRsText")?;
    println!("{}", session.fetch_string(stored)?);

    session.logout()?;
    Ok(())
}
```

Class-browser operations are available through the reusable library API used by
both the CLI and explorer:

```rust
use gemstone_rs::{browser::Browser, Config, Session};

let config = Config::from_env()?;
let mut session = Session::login(config)?;
let mut browser = Browser::new(&mut session);

let dictionaries = browser.dictionaries()?;
let classes = browser.classes("UserGlobals")?;
let protocols = browser.protocols("Object", false, "")?;
let methods = browser.methods("Object", "-- all --", false, "")?;
let source = browser.source("Object", "printString", false, "")?;
```

Web-service helpers are dependency-free and are reused by the standard HTTP,
Axum, and Actix examples. Use `gemstone-rs-axum` or `gemstone-rs-actix` when
you want framework routes without copying handler code:

```rust
use gemstone_rs::{web, Config, SessionWorkerPool};

let pool = SessionWorkerPool::start(Config::from_env()?, 2)?;
let response = web::gemstone_health_response(&pool);
assert_eq!(response.status, 200);
pool.shutdown()?;
```

The worker pool also has awaitable calls for async runtimes. The future wakes
when the worker thread finishes, while `Session` stays on the thread that
logged in:

```rust,no_run
use gemstone_rs::{Config, SessionWorkerPool, Value};

let pool = SessionWorkerPool::start(Config::from_env()?, 2)?;
assert_eq!(pool.eval_async("3 + 4").await?, Value::SmallInt(7));
pool.shutdown()?;
# Ok::<(), gemstone_rs::Error>(())
```

```rust,no_run
let pool = gemstone_rs_axum::pool_from_env(2)?;
let app = gemstone_rs_axum::router_with_name(pool, "booking service");
```

Use `gemstone_rs_axum::router_from_env_with_name(2, "booking service")` or
`gemstone_rs_actix::scope_from_env_with_name(2, "booking service")` when the
HTTP process should start before GemStone credentials are configured. `/` and
`/health/local` keep working; `/health/gemstone` returns a `503` JSON error
until the pool is available.

The framework adapters also emit `x-gemstone-rs-adapter`,
`x-gemstone-rs-route`, `x-gemstone-rs-request-id`,
`x-gemstone-rs-request-method`, `x-gemstone-rs-request-path`,
`x-gemstone-rs-request-lifecycle`, and
`x-gemstone-rs-request-duration-us` headers so route smoke tests, proxies, and
logs can identify which adapter, route, request, and handler lifecycle produced
the response. The checked Axum and Actix services also add application-level
middleware headers: `x-gemstone-rs-example-middleware`,
`x-gemstone-rs-service`, `x-gemstone-rs-service-version`,
`cache-control: no-store`, and `x-content-type-options: nosniff`. That proves
packaged routes still compose with normal framework middleware and shows a
small production-style cache/security/header policy. If the caller sends
`x-request-id`, the adapters propagate it into `x-gemstone-rs-request-id`;
otherwise they generate a local adapter-scoped id.

Runtime environment:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

## Examples

```bash
cargo run -p gemstone-rs --example hello_gemstone
cargo run -p gemstone-rs --example quickstart
cargo run -p gemstone-rs --example browser
cargo run -p gemstone-rs --example live_smoke_cookbook
cargo run -p gemstone-rs --example transactions
cargo run -p gemstone-rs --example session_worker
cargo run -p gemstone-rs --example session_worker_pool
cargo run -p gemstone-rs --example async_worker
cargo run -p gemstone-rs --example python_native_adapter -- --dry-run
cargo run -p gemstone-rs --example oop_values
cargo run -p gemstone-rs --example bridge_root_mapping
cargo run -p gemstone-rs --example bridge_value_inspection
cargo run -p gemstone-rs --example codegen_preview
cargo run -p gemstone-rs --example codegen_workflow
cargo run -p gemstone-rs --example generated_wrapper_app
cargo run -p gemstone-rs --example generated_mapping_app
cargo run -p gemstone-rs --example http_service -- --routes
cargo run -p gemstone-rs --example codegen_discover
cargo run --manifest-path examples/axum-service/Cargo.toml -- --routes
cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes
python3 scripts/framework_route_smoke.py
python3 scripts/framework_route_smoke.py --live
```

Additional walkthroughs:

- [CLI browser walkthrough](examples/tooling/cli-browser-walkthrough.md)
- [Standard-library HTTP service example](crates/gemstone-rs/examples/http_service.rs)
- [Session worker pool example](crates/gemstone-rs/examples/session_worker_pool.rs)
- [Async worker facade example](crates/gemstone-rs/examples/async_worker.rs)
- [Python native adapter contract example](crates/gemstone-rs/examples/python_native_adapter.rs)
- [Checked Axum service example](examples/axum-service/README.md)
- [Checked Actix service example](examples/actix-service/README.md)

See [examples/README.md](examples/README.md) and
[docs/examples-guide.md](docs/examples-guide.md) for the full map.

## CLI

```bash
cargo run -p gemstone-rs-cli -- doctor
cargo run -p gemstone-rs-cli -- doctor --live
cargo run -p gemstone-rs-cli -- doctor --strict
cargo run -p gemstone-rs-cli -- doctor --env-file .env.gemstone-rs --live
cargo run -p gemstone-rs-cli -- doctor --json
cargo run -p gemstone-rs-cli -- hello
cargo run -p gemstone-rs-cli -- hello --json
cargo run -p gemstone-rs-cli -- compare gemstone-py
cargo run -p gemstone-rs-cli -- compare gemstone-py --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --status
cargo run -p gemstone-rs-cli -- compare gemstone-py --status --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --scorecard
cargo run -p gemstone-rs-cli -- compare gemstone-py --scorecard --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --parity
cargo run -p gemstone-rs-cli -- compare gemstone-py --parity --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --gaps
cargo run -p gemstone-rs-cli -- compare gemstone-py --gaps --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --next
cargo run -p gemstone-rs-cli -- compare gemstone-py --next --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --totals
cargo run -p gemstone-rs-cli -- compare gemstone-py --totals --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --batches
cargo run -p gemstone-rs-cli -- compare gemstone-py --batches --json
cargo run -p gemstone-rs-cli -- compare all
cargo run -p gemstone-rs-cli -- compare all --json
cargo run -p gemstone-rs-cli -- compare all --status
cargo run -p gemstone-rs-cli -- compare all --status --json
cargo run -p gemstone-rs-cli -- compare all --scorecard
cargo run -p gemstone-rs-cli -- compare all --scorecard --json
cargo run -p gemstone-rs-cli -- compare all --parity
cargo run -p gemstone-rs-cli -- compare all --parity --json
cargo run -p gemstone-rs-cli -- compare all --gaps
cargo run -p gemstone-rs-cli -- compare all --gaps --json
cargo run -p gemstone-rs-cli -- compare all --next
cargo run -p gemstone-rs-cli -- compare all --next --json
cargo run -p gemstone-rs-cli -- compare all --totals
cargo run -p gemstone-rs-cli -- compare all --totals --json
cargo run -p gemstone-rs-cli -- compare all --batches
cargo run -p gemstone-rs-cli -- compare all --batches --json
cargo run -p gemstone-rs-cli -- env sample
cargo run -p gemstone-rs-cli -- env write .env.gemstone-rs
cargo run -p gemstone-rs-cli -- examples list
cargo run -p gemstone-rs-cli -- examples map
cargo run -p gemstone-rs-cli -- examples show quickstart
cargo run -p gemstone-rs-cli -- examples run codegen_preview --dry-run
cargo run -p gemstone-rs-cli -- examples scaffold quickstart /tmp/gemstone-rs-quickstart --force
cargo run -p gemstone-rs-cli -- examples scaffold browser /tmp/gemstone-rs-browser --force
cargo run -p gemstone-rs-cli -- examples scaffold bridge_root_mapping /tmp/gemstone-rs-bridge-root-mapping --force
cargo run -p gemstone-rs-cli -- examples scaffold derive_mapping /tmp/gemstone-rs-derive-mapping --force
cargo run -p gemstone-rs-cli -- examples scaffold codegen_preview /tmp/gemstone-rs-codegen-preview --force
cargo run -p gemstone-rs-cli -- examples scaffold codegen_workflow /tmp/gemstone-rs-codegen-workflow --force
cargo run -p gemstone-rs-cli -- examples scaffold codegen_discover /tmp/gemstone-rs-codegen-discover --force
cargo run -p gemstone-rs-cli -- examples scaffold codegen_discover_mapping /tmp/gemstone-rs-codegen-discover-mapping --force
cargo run -p gemstone-rs-cli -- examples scaffold profile_codegen_workflow /tmp/gemstone-rs-profile-codegen-workflow --force
cargo run -p gemstone-rs-cli -- examples scaffold generated_wrapper_app /tmp/gemstone-rs-generated-wrapper-app --force
cargo run -p gemstone-rs-cli -- examples scaffold generated_mapping_app /tmp/gemstone-rs-generated-mapping-app --force
cargo run -p gemstone-rs-cli -- examples scaffold http_service /tmp/gemstone-rs-http-service --force
cargo run -p gemstone-rs-cli -- examples scaffold axum_service /tmp/gemstone-rs-axum-service --force
cargo run -p gemstone-rs-cli -- examples scaffold actix_service /tmp/gemstone-rs-actix-service --force
cargo run -p gemstone-rs-cli -- examples run axum_service --dry-run -- --routes
cargo run -p gemstone-rs-cli -- examples run actix_service --dry-run -- --routes
cargo run -p gemstone-rs-cli -- eval --env-file .env.gemstone-rs "3 + 4"
cargo run -p gemstone-rs-cli -- browse dictionaries
cargo run -p gemstone-rs-cli -- browse classes UserGlobals
cargo run -p gemstone-rs-cli -- browse protocols Object
cargo run -p gemstone-rs-cli -- browse methods Object "-- all --"
cargo run -p gemstone-rs-cli -- browse source Object printString
cargo run -p gemstone-rs-cli -- inspect oop 20
cargo run -p gemstone-rs-cli -- bridge root
cargo run -p gemstone-rs-cli -- bridge keys
cargo run -p gemstone-rs-cli -- bridge get BookingDraft --symbol
cargo run -p gemstone-rs-cli -- bridge inspect BookingDraft --symbol
cargo run -p gemstone-rs-cli -- bridge put WorkbenchDraft "hello from Rust" --type String
cargo run -p gemstone-rs-cli -- bridge put WorkbenchCount 7 --type SmallInt
cargo run -p gemstone-rs-cli -- bridge remove WorkbenchDraft
cargo run -p gemstone-rs-cli -- bridge sample-config BookingDraft
cargo run -p gemstone-rs-cli -- codegen init
cargo run -p gemstone-rs-cli -- codegen preview examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen diff examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check
cargo run -p gemstone-rs-cli -- codegen explain examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen explain --json examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen generate examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen explain-profile --json default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen discover examples/codegen/discovered.codegen Object
```

Codegen method arguments can stay explicit as OOPs or be typed at the wrapper
edge. For example, `args=id:SmallInt,selector:Symbol,name:String,active:Bool`
generates Rust parameters such as `id: i64`, `selector: impl AsRef<str>`, and
`active: bool`, then converts them with the active `Session` before calling
GemStone. This keeps the generated wrappers easier to call from Rust services
without hiding the lower-level OOP API.

The CLI intentionally uses only the standard library. `doctor` checks the
GemStone environment, GCI library resolution, and optionally a live `3 + 4`
probe. The report includes which source selected `libgcirpc`: explicit config,
`GS_LIB_PATH`, `GS_LIB`, or `GEMSTONE/lib`, plus the path or directory searched.
When setup fails, it prints actionable hints for credentials, library loading,
or live stone connectivity. Add `--json` when release scripts or VS Code need
structured output. `env sample` prints a safe shell export template for the
same `GS_*` variables and replaces password values with placeholders.
`env write` writes that template to `.env.gemstone-rs` or a chosen path and
refuses to overwrite unless `--force` is passed. `--env-file` works as a global
CLI option, so `doctor`, `eval`, `browse`, `inspect`, `bridge`, and `codegen`
commands can load that file without requiring the caller to source it in the
shell. `gemstone-rs-explorer --env-file .env.gemstone-rs` uses the same file
for local browser workflows. `doctor --strict` is intended for CI: it fails when the stone or GCI
library source is only coming from defaults. GCI diagnostics also report
whether the selected `libgcirpc` exists, is a file, is readable, and whether
the path appears to be arm64 or x86_64.
`hello` is a no-GemStone sanity check similar to `gemstone-examples hello`; it
prints the CLI version, target OS, target architecture, and executable path.
`compare gemstone-py` prints the compact Rust/Python comparison from the
comparison guide, with `--json` for tooling. The structured compare output is
documented in `schemas/gemstone-rs.compare.schema.json` and covered by the
schema validation script. Add `--gaps` when you want the actionable catch-up
list: what gemstone-py already does better, what gemstone-rs still lacks, the
next implementation action, and the verification command for each gap.
`examples list` and `examples show <name>` provide an installed-CLI example
index similar to `gemstone-examples list` in gemstone-py; `examples run <name>`
launches the selected Cargo example from a source checkout, and `--dry-run`
prints the command without executing it. `examples scaffold <name> [path]`
creates a standalone Cargo project from an installed template, including
`quickstart`, `browser`, `bridge_root_mapping`, `derive_mapping`,
`codegen_preview`, `codegen_workflow`, `codegen_discover`,
`codegen_discover_mapping`, `profile_codegen_workflow`,
`generated_wrapper_app`, `generated_mapping_app`, `http_service`,
`session_worker_pool`, `axum_service`, and `actix_service`, so users can try
gemstone-rs without keeping the repository checkout open. The source checkout
also includes `bridge_value_inspection`, which reads nested BridgeRoot
dictionaries and arrays back as dynamic `BridgeValue` trees before a typed
mapping is finalized, and `bridge_mapping_preview`, which infers a reviewable
starter `BridgeMapped` codegen config from that dynamic shape.
`profile_codegen_workflow` also writes
`gemstone-rs.codegen` and `gemstone-rs.codegen-profiles.json` beside
`src/main.rs`. `examples map` mirrors the `gemstone-examples plan3-map` idea
by showing which Rust crates, examples, and docs correspond to each feature
stream and gemstone-py reference point. The JSON forms are used by tooling and
are suitable for CI checks.
`eval`, `inspect oop`, and `bridge` commands are wired to live GemStone calls.
`bridge keys` lists the keys currently stored under `GemStoneRsBridgeRoot`,
`bridge value <key> --depth 4` prints a nested dynamic `BridgeValue` tree,
`bridge shape <key> --depth 4` prints relationship paths and node counts,
`bridge mapping-preview <key> --mapped BookingDraft --depth 4` converts a live
BridgeRoot value into a reviewable mapping config, and `bridge put` and
`bridge remove` make explicit committed BridgeRoot edits.
The `browse` commands cover dictionaries, classes, protocols, methods, and
source using the active user's symbol list. The `codegen` commands read a
line-oriented config, explain what will be generated, preview generated Rust
wrappers, diff/check stale output, generate configs from a live stone, and
write generated files.

## Codegen

Start with:

```bash
cargo run -p gemstone-rs-cli -- codegen init
```

Config example:

```text
output = src/generated/gemstone_wrappers.rs
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

Optional metadata controls generated signatures and docs:

```text
method = UserGlobals:OkzBooking>>findById: | args=id | return=Oop | doc=Find a booking by id.
method = Object>>printString | return=String | doc=Return the receiver printString.
```

See `examples/codegen/` for a concrete generated wrapper example.

## Explorer

The explorer is intentionally local-only and read-only by default:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
cargo run -p gemstone-rs-explorer -- --port 8787 --codegen-root /path/to/gemstone-rs
```

Open:

```text
http://127.0.0.1:8787/
http://127.0.0.1:8787/api/status
http://127.0.0.1:8787/api/compare/gemstone-py/status
http://127.0.0.1:8787/api/compare/all/status
http://127.0.0.1:8787/api/browse/dictionaries
http://127.0.0.1:8787/api/browse/classes?dictionary=UserGlobals
http://127.0.0.1:8787/api/browse/protocols?class=Object
http://127.0.0.1:8787/api/browse/methods?class=Object&protocol=--%20all%20--
http://127.0.0.1:8787/api/browse/source?class=Object
http://127.0.0.1:8787/api/codegen/sample
http://127.0.0.1:8787/api/codegen/configs?root=.
http://127.0.0.1:8787/api/codegen/profiles?profile_file=gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/codegen/profiles/check?profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/codegen/config?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/discover-mapping?mapped=BookingDraft&class=Object
http://127.0.0.1:8787/api/codegen/explain?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/explain-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/preview-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/diff-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/codegen/check?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/check-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/bridge/root
http://127.0.0.1:8787/api/bridge/keys
http://127.0.0.1:8787/api/inspect?oop=20
```

Workspace eval is opt-in:

```bash
cargo run -p gemstone-rs-explorer -- --allow-eval
```

The explorer uses standard-library HTTP only. The home page exposes the main
browse, BridgeRoot, codegen, and comparison-status workflows directly. It can
load/save the selected codegen config file through a POST body when write mode
is enabled, lists known `.codegen` files through a project-aware picker, keeps
a local recent-config history, saves named local codegen profiles,
exports/imports profile JSON, loads/saves project profile files with schema
validation, renders generated source, generated config, profile-aware
preview/diff/check/explain summaries, whole-project profile status, unified
diff, side-by-side diff output, and the gemstone-rs vs gemstone-py batch
status in a dedicated detail pane, remembers the current fields locally, and
keeps the JSON endpoints stable for curl, VS Code, and automation.

Generate endpoints are write-gated:

```bash
cargo run -p gemstone-rs-explorer -- --allow-write
```

```text
http://127.0.0.1:8787/api/codegen/generate?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/generate-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/codegen/profiles/save?profile_file=gemstone-rs.codegen-profiles.json
```

Project profile samples live at:

```text
examples/codegen/gemstone-rs.codegen-profiles.json
```

Create, inspect, and validate profile files from the CLI:

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

The schema and validation guide live at [docs/profile-schema.md](docs/profile-schema.md),
with the JSON Schema at
[schemas/gemstone-rs.codegen-profiles.schema.json](schemas/gemstone-rs.codegen-profiles.schema.json).

Write endpoints reject `..` traversal in `config=`, `profile_file=`, and
`root=` after URL decoding, keep relative writes under the configured codegen
root, and require
`--allow-absolute-write-paths` before absolute write targets are accepted.
Project profile saves also validate the schema before writing: only `kind`,
`version`, and `profiles` are allowed at the top level, profile names must be
present and unique, and profile fields must be string-valued.

For config saves, send the config as the request body:

```bash
curl -s -X POST \
  --data-binary @examples/codegen/gemstone-rs.codegen \
  'http://127.0.0.1:8787/api/codegen/config/save?config=examples/codegen/draft.codegen'
```

BridgeRoot write endpoints are also write-gated:

```bash
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchDraft&value=hello'
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchCount&value=7&value_type=SmallInt'
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchFlag&value=true&value_type=Bool&key_type=Symbol'
curl -s 'http://127.0.0.1:8787/api/bridge/remove?key=WorkbenchDraft'
```

## VS Code

`vscode-gemstone-rs-workbench` provides command palette actions that call the
Rust CLI and open output/preview editors:

```json
{
  "gemstoneRs.checkoutPath": "/path/to/gemstone-rs",
  "gemstoneRs.useCargo": true,
  "gemstoneRs.codegenConfig": "examples/codegen/gemstone-rs.codegen"
}
```

Commands include setup verification, eval, browse dictionaries/classes,
codegen init/discover/preview/diff/check/explain/generate, launch explorer,
and open an embedded explorer webview. The GemStone RS activity bar view
browses dictionaries, classes, protocols, methods, and the configured codegen
actions. `Codegen Explain` renders the structured classes, selectors, mapped
fields, and generated test stubs. The embedded webview can now preview, edit,
open, and save generated wrapper output through VS Code with a confirmation
prompt. `Codegen Generate` still shows the generated diff before writing files.

## Threading

`Session` is deliberately not `Send` or `Sync`. Keep a session on the thread
that logged it in until GemStone GCI threading behavior is proven safe for
broader sharing.

## CI

The repository CI runs:

```bash
make verify
```

`make verify` runs Rust formatting, `cargo check`, clippy, tests, the codegen
freshness check for `examples/codegen/gemstone-rs.codegen`, the VS Code
extension syntax check, the VS Code smoke check, and PDF generation.

Package the VS Code extension locally with:

```bash
make vscode-package
```

That writes `vscode-gemstone-rs-workbench/gemstone-rs-workbench-0.3.4.vsix`.
The generated `.vsix` and `node_modules/` are intentionally ignored.

Verify published artifacts with:

```bash
scripts/publish_verify.sh 0.2.2
```

The verification script checks crates.io package versions, installs
all published crates, installs `gemstone-rs-cli` and `gemstone-rs-explorer`,
runs both binaries with `--help`, and confirms the Marketplace version matches
the VS Code package metadata.

## Publishing

The crates must be published in dependency order:

```bash
cargo publish -p gemstone-gci
cargo publish -p gemstone-rs-macros
cargo publish -p gemstone-rs
cargo publish -p gemstone-rs-axum
cargo publish -p gemstone-rs-actix
cargo publish -p gemstone-rs-cli
cargo publish -p gemstone-rs-explorer
```

Or use the repository helper:

```bash
scripts/publish_crates.sh
DRY_RUN=1 scripts/publish_crates.sh
```

Before `gemstone-gci` is published, `cargo package --workspace` is expected to
fail when it validates `gemstone-rs` against the crates.io index. Verify the
leaf crate first with:

```bash
cargo package -p gemstone-gci --no-verify
```

Run the opt-in live smoke lane with:

```bash
scripts/live_smoke.sh --dry-run
scripts/live_smoke.sh
```

The script checks the required GemStone environment up front, then runs
`doctor --strict --live`, the serial Rust live tests, `live_smoke_cookbook`,
`python_native_adapter`, and the live Axum/Actix route smoke. The underlying
Rust test command still uses `--test-threads=1` to avoid concurrent GCI
sessions inside one test process.

## Explorer Roadmap

The first browse, BridgeRoot, codegen, project-aware config picker/load/save,
recent config history, named local profiles, diff detail, local field
persistence, project profile file load/save, editable generated-output webview,
browser fallback paths, nested BridgeValue rendering, and committed
Marketplace/GitHub visuals are now wired. Shape reports now include repeated
OOP identity groups, so the explorer can show relationship paths for the same
GemStone object across a nested payload. Next explorer work should focus on
deeper generated-file editing and richer live object navigation over the stable
local API.
