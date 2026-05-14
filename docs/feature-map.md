# gemstone-rs Feature Map

This map is the Rust equivalent of `gemstone-examples plan3-map` in
`gemstone-py`: it ties each feature stream to the crates, examples, docs, and
Python reference point that inspired it.

Run it from the CLI:

```bash
gemstone-rs hello
gemstone-rs compare gemstone-py
gemstone-rs compare gemstone-py --gaps
gemstone-rs examples map
gemstone-rs examples map --json
gemstone-rs examples scaffold quickstart ./gemstone-rs-quickstart
gemstone-rs examples scaffold codegen_workflow ./gemstone-rs-codegen-workflow
gemstone-rs examples scaffold profile_codegen_workflow ./gemstone-rs-profile-codegen
gemstone-rs examples scaffold generated_wrapper_app ./gemstone-rs-generated-wrapper
gemstone-rs examples scaffold session_worker_pool ./gemstone-rs-worker-pool
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
```

`hello` is the no-live sanity check that mirrors `gemstone-examples hello`.
`compare gemstone-py` prints a compact version of the Python/Rust comparison
guide. `compare gemstone-py --gaps` prints the remaining parity gaps with next
actions and verification commands. `examples scaffold` creates standalone Cargo
projects from installed templates for quickstart, browser, BridgeRoot mapping,
derive mapping, generated wrappers, live discovery, profile-driven codegen,
standard HTTP, worker-pool, Axum, and Actix workflows. JSON output is intended for CI, docs
checks, and editor tooling.

## Streams

| Stream | Feature | Rust surface | Examples | Docs | gemstone-py reference | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Runtime and GCI loading | `gemstone-gci`, `gemstone-rs::Config` | `hello`, `hello_gemstone`, `quickstart` | `docs/setup-guide.md`, `docs/performance-safety.md` | `gemstone_py.native`, native backend checks | Rust-native core; Python still has broader packaged backend docs. |
| 2 | Safe sessions and transactions | `gemstone-rs::Session`, `SessionWorker`, `SessionWorkerPool`, `TransactionGuard` | `quickstart`, `transactions`, `session_worker`, `session_worker_pool`, `live_smoke_cookbook` | `docs/user-manual.md`, `docs/cookbook.md` | `GemStoneSession`, `SessionFacade`, transaction policies | Core parity for sync eval, perform, commit, and abort, plus dedicated-thread workers and a bounded worker pool. |
| 3 | Browser and inspection | `gemstone-rs::browser`, CLI `browse`, `gemstone-rs-explorer` | `browser`, `tooling/cli-browser-walkthrough.md` | `docs/user-manual.md`, `docs/explorer.md` | `gemstone_py.inspection`, `python-gemstone-database-explorer` | API parity is growing; the Python explorer is still more mature. |
| 4 | OOP and value handling | `Oop`, `Value`, export-set helpers | `oop_values` | `docs/user-manual.md`, `docs/performance-safety.md` | Managed OOP handles and typed access examples | Rust has an explicit ownership model; Python is easier for casual scripting. |
| 5 | BridgeRoot object mapping | `gemstone-rs::bridge`, `gemstone-rs-macros` | `bridge_root_mapping`, `derive_mapping`, `generated_mapping_app` | `docs/object-mapping.md`, `docs/cookbook.md` | `SmalltalkBridge`, `PersistentRoot`, facade examples | Rust has typed mapping and derive; a transparent object model remains future work. |
| 6 | Typed codegen | `gemstone-rs::codegen`, CLI `codegen` and `profile` | `codegen_preview`, `codegen_workflow`, `codegen_discover`, `profile_codegen_workflow`, `generated_wrapper_app` | `docs/codegen.md`, `docs/profile-schema.md` | `gemstone_py.codegen`, `typed_access/codegen_demo` | Preview, diff, check, generate, and profile workflow parity; live discovery still needs more depth. |
| 7 | Explorer workflow | `gemstone-rs-explorer` | `tooling/explorer.md` | `docs/explorer.md`, `docs/screenshots.md` | `python-gemstone-database-explorer` | Useful local UI/API; Python explorer remains the richer product reference. |
| 8 | VS Code workbench | `vscode-gemstone-rs-workbench` | `tooling/vscode-workbench.md` | `docs/vscode-workbench.md` | `gemstone-py Workbench` | Command and webview workflow exists; embedded explorer UX needs polish. |
| 9 | Rust web services | `gemstone-rs::web`, `gemstone-rs-axum`, `gemstone-rs-actix`, `SessionWorkerPool`, checked Axum/Actix services, and installed scaffolds | `session_worker`, `session_worker_pool`, `http_service`, `examples/axum-service`, `examples/actix-service`, `framework_route_smoke.py`, `examples scaffold session_worker_pool/axum_service/actix_service` | `docs/examples-guide.md`, `docs/cookbook.md` | FastAPI, Litestar, Django examples | Shared JSON health helpers, standard-library HTTP, graceful health-pool startup, diagnostic headers, packaged Axum/Actix adapters, checked services, and route smoke coverage exist; richer middleware remains planned. |
| 10 | Release and verification | `scripts`, `Makefile`, GitHub Actions | Release verification commands | `docs/release-checklist.md` | PyPI/TestPyPI/native wheel/VSIX release tooling | Crates/VSIX verification path exists; Python release lane is more complete. |
| 11 | Shared native core | `gemstone-gci`, `gemstone-rs`, future `gemstone-py-native` wrapper | Shared-core integration plan | `docs/shared-core-integration.md` | `gemstone-py-native` | Best long-term architecture; not wired into `gemstone-py-native` yet. |

## What This Says Compared With gemstone-py

`gemstone-rs` is strongest where Rust is naturally valuable: direct native
GemStone access, explicit OOP/value handling, typed generated wrappers,
BridgeRoot mapping, and CLI/explorer tooling that can run without Python in
the process.

`gemstone-py` remains stronger where Python already has mature product shape:
broader web framework adapters, async examples, the database explorer, package
extras, and broader release surfaces.

The projects should converge by sharing the Rust native core underneath
`gemstone-py-native`, while keeping the Python and Rust APIs idiomatic for
their respective users.
