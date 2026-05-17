# gemstone-rs Feature Map

This map is the Rust equivalent of `gemstone-examples plan3-map` in
`gemstone-py`: it ties each feature stream to the crates, examples, docs, and
Python reference point that inspired it.

Run it from the CLI:

```bash
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
guide. `compare gemstone-js` remains available as archived background
reference, but active planning now focuses on gemstone-rs. The `--status`
forms give the shortest answer with the direct recommendation, parity score,
remaining batch count, next batch, top gap, and follow-up commands. The `--scorecard`
forms print the shortest decision view: when to use each project, current
strengths, the remaining batch count, the next batch, and the top gap. The
`--parity` forms print area-by-area maturity scores for API, web, async,
codegen, explorer, release, and native-core work. The `--gaps`
forms print remaining parity gaps with next actions and verification commands.
The `--next` forms print the first recommended batch and top priority gap.
The `--totals` forms print only the batch/hour totals for planning and CI.
The `--batches` forms answer how much work remains, including batch counts,
hour ranges, outcomes, and verification commands. Use `compare all` with the
same flags to print the active gemstone-rs track; `compare all --batches`
reports **2 batches** and roughly **10-17 hours**.
`examples scaffold` creates standalone Cargo
projects from installed templates for quickstart, browser, BridgeRoot mapping,
derive mapping, generated wrappers, live discovery, profile-driven codegen,
standard HTTP, worker-pool, Axum, and Actix workflows. JSON output is intended for CI, docs
checks, and editor tooling. Compare reports are covered by
[`schemas/gemstone-rs.compare.schema.json`](../schemas/gemstone-rs.compare.schema.json),
so release scripts and editor panels can validate the same summary, status,
scorecard, parity, gap, next-action, totals, and batch-plan shapes that the CLI
prints.

## Streams

| Stream | Feature | Rust surface | Examples | Docs | gemstone-py reference | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Runtime and GCI loading | `gemstone-gci`, `gemstone-rs::Config` | `hello`, `hello_gemstone`, `quickstart` | `docs/setup-guide.md`, `docs/performance-safety.md` | `gemstone_py.native`, native backend checks | Rust-native core; Python still has broader packaged backend docs. |
| 2 | Safe sessions and transactions | `gemstone-rs::Session`, `SessionWorker`, `SessionWorkerPool`, `TransactionGuard` | `quickstart`, `transactions`, `session_worker`, `session_worker_pool`, `live_smoke_cookbook` | `docs/user-manual.md`, `docs/cookbook.md` | `GemStoneSession`, `SessionFacade`, transaction policies | Core parity for sync eval, perform, commit, and abort, plus dedicated-thread workers and a bounded worker pool. |
| 3 | Browser and inspection | `gemstone-rs::browser`, CLI `browse`, `gemstone-rs-explorer` | `browser`, `tooling/cli-browser-walkthrough.md` | `docs/user-manual.md`, `docs/explorer.md` | `gemstone_py.inspection`, `python-gemstone-database-explorer` | API parity is growing; the Python explorer is still more mature. |
| 4 | OOP and value handling | `Oop`, `Value`, export-set helpers | `oop_values` | `docs/user-manual.md`, `docs/performance-safety.md` | Managed OOP handles and typed access examples | Rust has an explicit ownership model; Python is easier for casual scripting. |
| 5 | BridgeRoot object mapping | `gemstone-rs::bridge`, `gemstone-rs-macros` | `bridge_root_mapping`, `derive_mapping`, `bridge_value_inspection`, `generated_mapping_app` | `docs/object-mapping.md`, `docs/cookbook.md` | `SmalltalkBridge`, `PersistentRoot`, facade examples | Rust has typed mapping, derive, nested dynamic `BridgeValue` read-back, repeated-OOP identity groups, and path-aware diagnostics; a fully transparent object model remains future work. |
| 6 | Typed codegen | `gemstone-rs::codegen`, CLI `codegen` and `profile` | `codegen_preview`, `codegen_workflow`, `codegen_discover`, `profile_codegen_workflow`, `generated_wrapper_app` | `docs/codegen.md`, `docs/profile-schema.md` | `gemstone_py.codegen`, `typed_access/codegen_demo` | Preview, diff, check, generate, profiles, typed return helpers, typed argument conversion, generated metadata tests, and live discovery of source-header args plus protocol/source docs; deeper live type discovery remains planned. |
| 7 | Explorer workflow | `gemstone-rs-explorer` | `tooling/explorer.md` | `docs/explorer.md`, `docs/screenshots.md` | `python-gemstone-database-explorer` | Useful local UI/API with setup, profile status, live browse path/source previews, editable generated output, nested BridgeValue rendering, codegen workflows, VS Code open-file actions, browser fallback prompts, and committed visual assets; Python explorer remains the richer product reference. |
| 8 | VS Code workbench | `vscode-gemstone-rs-workbench` | `tooling/vscode-workbench.md` | `docs/vscode-workbench.md` | `gemstone-py Workbench` | Command workflow and embedded webview now render setup checks, profile status, codegen summaries/diffs/editable output files, BridgeRoot keys/nested values, comparison status, and Marketplace/GitHub visuals. |
| 9 | Rust web services | `gemstone-rs::web`, `gemstone-rs-axum`, `gemstone-rs-actix`, `SessionWorkerPool`, async worker futures, checked Axum/Actix services, and installed scaffolds | `session_worker`, `session_worker_pool`, `async_worker`, `http_service`, `examples/axum-service`, `examples/actix-service`, `framework_route_smoke.py`, `examples scaffold session_worker_pool/axum_service/actix_service` | `docs/examples-guide.md`, `docs/cookbook.md` | FastAPI, Litestar, Django examples | Shared JSON health helpers, standard-library HTTP, graceful health-pool startup, diagnostic, request-trace, lifecycle, duration, production-style service/cache/security middleware headers, packaged Axum/Actix adapters, checked services, async health handlers, local/live route smoke coverage, installed scaffolds, and a dependency-free async worker facade exist. |
| 10 | Release and verification | `scripts`, `Makefile`, GitHub Actions | Release verification commands | `docs/release-checklist.md` | PyPI/TestPyPI/native wheel/VSIX release tooling | Crates/VSIX verification path exists; Python release lane is more complete. |
| 11 | Shared native core | `gemstone-gci`, `gemstone-rs::py_native`, future `gemstone-py-native` wrapper | `python_native_adapter`, shared-core integration plan | `docs/shared-core-integration.md` | `gemstone-py-native` | Rust-side PyO3 adapter contract exists; `gemstone-py-native` still needs to wrap it. |

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
