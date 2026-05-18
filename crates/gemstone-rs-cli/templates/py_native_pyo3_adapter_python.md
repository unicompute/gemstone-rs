# gemstone-py-native PyO3 Starter

This scaffold is intentionally small. It demonstrates the intended long-term
shape for `gemstone-py-native`:

```text
Python -> PyO3 classes/functions -> gemstone_rs::py_native -> Session -> GCI
```

Build it locally:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install maturin pytest
maturin develop
python -c 'import gemstone_py_native; print(gemstone_py_native.capabilities_json())'
python -c 'import gemstone_py_native; print(gemstone_py_native.samples_json())'
python -c 'import gemstone_py_native; print(gemstone_py_native.migration_json())'
python -c 'import gemstone_py_native; print(gemstone_py_native.compatibility_json())'
python -c 'import gemstone_py_native; print(gemstone_py_native.conformance_json())'
python -c 'import gemstone_py_native; print(gemstone_py_native.handoff_json())'
python -c 'import gemstone_py_native_compat; print(gemstone_py_native_compat.compatibility_report()["returnPolicy"])'
pytest
```

`migration_json()` exposes the same shared-core checklist as
`gemstone-rs py-native migration --json`. It is useful in Python wrapper CI and
release notes because it names the remaining steps for making
`gemstone-py-native` a thin PyO3 wrapper over `gemstone_rs::py_native`.
`compatibility_json()` exposes the generated Python shim contract from
`gemstone-rs py-native compatibility --json`: every compatibility method, the
underlying native method, and the Python return type.
`conformance_json()` exposes the wrapper conformance target from
`gemstone-rs py-native conformance --json`: required module functions,
`NativeSession` methods, compatibility shim methods, fixture checks, and
scaffold files.
`handoff_json()` exposes the final downstream handoff manifest from
`gemstone-rs py-native handoff --json`: artifact paths, schemas, regeneration
commands, validation commands, and release acceptance checks.

The generated `NativeSession` class exposes a deliberately direct adapter
surface over the Rust core: `eval_oop`, `execute`, `resolve`,
`eval_json`, `perform_raw_oop`, `perform_json`, `new_string`, `new_symbol`,
`fetch_string`, `value_to_oop_*`, `global_get`, `global_put_raw`,
`global_put_string`, `global_put_smallint`, export-set helpers, transaction
status, `commit`, `abort`, and `logout`. Keep Pythonic return conversion in
the Python package layer above this class.

The generated `gemstone_py_native_compat.py` file demonstrates that package
layer. It wraps raw native object identity in `OopHandle`, exposes
`NativeCompatibilitySession`, and leaves typed conversions as explicit opt-in
helpers. `eval_value()` and `perform_value()` decode the stable Rust
`PyNativeValue` JSON shape into dictionaries, while OOP-returning calls produce
`OopHandle`. This is the pattern to use when wiring the real
`gemstone-py-native` package to the Rust core without changing existing Python
return behavior by default.

Live smoke with GemStone credentials:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=your-password

python - <<'PY'
import gemstone_py_native_compat

session = gemstone_py_native_compat.NativeCompatibilitySession.login_from_env()
try:
    assert session.eval_smallint("3 + 4") == 7
finally:
    session.logout()
PY
```

Keep this adapter thin. Python package code can add Pythonic APIs, async
facades, packaging, and backward-compatible return behavior above this native
core.
