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
gemstone-rs compare gemstone-js
gemstone-rs compare gemstone-js --status
gemstone-rs compare gemstone-js --scorecard
gemstone-rs compare gemstone-js --parity
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
guide. `compare gemstone-js` prints the TypeScript/Python comparison from
[gemstone-js vs gemstone-py](gemstone-js-vs-gemstone-py.md). The `--status`
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
same flags to print both comparison tracks together; `compare all --batches`
reports **12 batches** and roughly **74-128 hours** across the Rust and
TypeScript catch-up tracks.
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
| 5 | BridgeRoot object mapping | `gemstone-rs::bridge`, `gemstone-rs-macros` | `bridge_root_mapping`, `derive_mapping`, `generated_mapping_app` | `docs/object-mapping.md`, `docs/cookbook.md` | `SmalltalkBridge`, `PersistentRoot`, facade examples | Rust has typed mapping and derive; a transparent object model remains future work. |
| 6 | Typed codegen | `gemstone-rs::codegen`, CLI `codegen` and `profile` | `codegen_preview`, `codegen_workflow`, `codegen_discover`, `profile_codegen_workflow`, `generated_wrapper_app` | `docs/codegen.md`, `docs/profile-schema.md` | `gemstone_py.codegen`, `typed_access/codegen_demo` | Preview, diff, check, generate, profiles, typed return helpers, and typed argument conversion; live discovery still needs more depth. |
| 7 | Explorer workflow | `gemstone-rs-explorer` | `tooling/explorer.md` | `docs/explorer.md`, `docs/screenshots.md` | `python-gemstone-database-explorer` | Useful local UI/API with setup, profile status, live browse path/source previews, editable generated output, BridgeRoot key/value summaries, codegen workflows, VS Code open-file actions, and browser fallback prompts; Python explorer remains the richer product reference. |
| 8 | VS Code workbench | `vscode-gemstone-rs-workbench` | `tooling/vscode-workbench.md` | `docs/vscode-workbench.md` | `gemstone-py Workbench` | Command workflow and embedded webview now render setup checks, profile status, codegen summaries/diffs/editable output files, BridgeRoot keys/values, and comparison status. |
| 9 | Rust web services | `gemstone-rs::web`, `gemstone-rs-axum`, `gemstone-rs-actix`, `SessionWorkerPool`, checked Axum/Actix services, and installed scaffolds | `session_worker`, `session_worker_pool`, `http_service`, `examples/axum-service`, `examples/actix-service`, `framework_route_smoke.py`, `examples scaffold session_worker_pool/axum_service/actix_service` | `docs/examples-guide.md`, `docs/cookbook.md` | FastAPI, Litestar, Django examples | Shared JSON health helpers, standard-library HTTP, graceful health-pool startup, diagnostic, request-trace, lifecycle, duration, production-style service/cache/security middleware headers, packaged Axum/Actix adapters, checked services, local/live route smoke coverage, and installed scaffolds exist; broader framework coverage and async facade remain planned. |
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
