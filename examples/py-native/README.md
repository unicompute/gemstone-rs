# gemstone-rs py-native Contract Fixture

This directory contains the checked-in JSON fixture for:

```bash
gemstone-rs py-native capabilities --json
```

The fixture is intentionally small and stable. It gives a future
`gemstone-py-native` PyO3 wrapper, CI job, or editor integration a concrete
contract sample without requiring a live GemStone/S stone.

Verify it from a source checkout:

```bash
cargo run -p gemstone-rs-cli -- py-native check examples/py-native/gemstone-rs.py-native.json
cargo run -p gemstone-rs-cli -- py-native check examples/py-native/gemstone-rs.py-native.json --json
node scripts/validate_codegen_schemas.js
```

Schema:

```text
schemas/gemstone-rs.py-native.schema.json
```
