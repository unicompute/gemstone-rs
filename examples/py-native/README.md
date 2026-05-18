# gemstone-rs py-native Contract Fixture

This directory contains the checked-in JSON fixture for:

```bash
gemstone-rs py-native capabilities --json
gemstone-rs py-native samples --json
gemstone-rs py-native smoke --dry-run --json
gemstone-rs py-native migration --json
gemstone-rs py-native compatibility --json
gemstone-rs py-native conformance --json
gemstone-rs py-native handoff --json
gemstone-rs py-native check-all --json
```

The fixtures are intentionally small and stable. They give a future
`gemstone-py-native` PyO3 wrapper, CI job, or editor integration a concrete
contract sample, value/error translation sample, and dry-run smoke report
without requiring a live GemStone/S stone. The migration report is not a
fixture because it describes the active `gemstone-py-native` integration plan:
wrap `PyNativeSession`, preserve Python return behavior, run live backend
smoke, and publish wheels after the Python native path is green. The
`py_native_pyo3_adapter` scaffold exposes the same report as
`gemstone_py_native.migration_json()` and includes
`python/gemstone_py_native_compat.py`, which demonstrates
`NativeCompatibilitySession`, `OopHandle`, and the backward-compatible return
policy that should live above the thin PyO3 module.
The compatibility fixture is checked in as
`gemstone-rs.py-native-compat.json`; it maps each generated Python shim method
to the underlying native method and expected Python return type.
The conformance fixture is checked in as
`gemstone-rs.py-native-conformance.json`; it lists the generated PyO3 module
functions, raw `NativeSession` methods, compatibility shim methods, fixture
paths, and scaffold files that downstream `gemstone-py-native` integration
should preserve.
The handoff fixture is checked in as `gemstone-rs.py-native-handoff.json`; it
bundles every py-native artifact, schema, regeneration command, validation
command, and downstream acceptance check needed before `gemstone-py-native`
uses the Rust core as its native backend.
The check-all report is covered by
`schemas/gemstone-rs.py-native-check-all.schema.json` and validates every
checked-in fixture in one downstream CI gate.

Verify it from a source checkout:

```bash
cargo run -p gemstone-rs-cli -- py-native check examples/py-native/gemstone-rs.py-native.json
cargo run -p gemstone-rs-cli -- py-native check examples/py-native/gemstone-rs.py-native.json --json
cargo run -p gemstone-rs-cli -- py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json
cargo run -p gemstone-rs-cli -- py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json --json
cargo run -p gemstone-rs-cli -- py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json
cargo run -p gemstone-rs-cli -- py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json --json
cargo run -p gemstone-rs-cli -- py-native smoke --dry-run
cargo run -p gemstone-rs-cli -- py-native smoke --dry-run --json
cargo run -p gemstone-rs-cli -- py-native migration --json
cargo run -p gemstone-rs-cli -- py-native compatibility --json
cargo run -p gemstone-rs-cli -- py-native check-compat examples/py-native/gemstone-rs.py-native-compat.json
cargo run -p gemstone-rs-cli -- py-native conformance --json
cargo run -p gemstone-rs-cli -- py-native check-conformance examples/py-native/gemstone-rs.py-native-conformance.json
cargo run -p gemstone-rs-cli -- py-native handoff --json
cargo run -p gemstone-rs-cli -- py-native check-handoff examples/py-native/gemstone-rs.py-native-handoff.json
cargo run -p gemstone-rs-cli -- py-native check-all
cargo run -p gemstone-rs-cli -- py-native check-all --json
cargo run -p gemstone-rs-cli -- examples run py_native_capabilities --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_contract_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_samples_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_smoke_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_migration_plan --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_compatibility_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_conformance_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_handoff_bundle --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_shared_core_gate --dry-run
node scripts/validate_codegen_schemas.js
```

Schema:

```text
schemas/gemstone-rs.py-native.schema.json
schemas/gemstone-rs.py-native-samples.schema.json
schemas/gemstone-rs.py-native-smoke.schema.json
schemas/gemstone-rs.py-native-migration.schema.json
schemas/gemstone-rs.py-native-compat.schema.json
schemas/gemstone-rs.py-native-conformance.schema.json
schemas/gemstone-rs.py-native-handoff.schema.json
schemas/gemstone-rs.py-native-check-all.schema.json
```
