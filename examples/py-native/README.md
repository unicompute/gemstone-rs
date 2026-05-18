# gemstone-rs py-native Contract Fixture

This directory contains the checked-in JSON fixture for:

```bash
gemstone-rs py-native capabilities --json
gemstone-rs py-native samples --json
gemstone-rs py-native smoke --dry-run --json
gemstone-rs py-native migration --json
```

The fixtures are intentionally small and stable. They give a future
`gemstone-py-native` PyO3 wrapper, CI job, or editor integration a concrete
contract sample, value/error translation sample, and dry-run smoke report
without requiring a live GemStone/S stone. The migration report is not a
fixture because it describes the active `gemstone-py-native` integration plan:
wrap `PyNativeSession`, preserve Python return behavior, run live backend
smoke, and publish wheels after the Python native path is green. The
`py_native_pyo3_adapter` scaffold exposes the same report as
`gemstone_py_native.migration_json()`.

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
cargo run -p gemstone-rs-cli -- examples run py_native_capabilities --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_contract_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_samples_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_smoke_fixture --dry-run
cargo run -p gemstone-rs-cli -- examples run py_native_migration_plan --dry-run
node scripts/validate_codegen_schemas.js
```

Schema:

```text
schemas/gemstone-rs.py-native.schema.json
schemas/gemstone-rs.py-native-samples.schema.json
schemas/gemstone-rs.py-native-smoke.schema.json
schemas/gemstone-rs.py-native-migration.schema.json
```
