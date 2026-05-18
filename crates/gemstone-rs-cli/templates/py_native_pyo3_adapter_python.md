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
pytest
```

Live smoke with GemStone credentials:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=your-password

python - <<'PY'
import gemstone_py_native

session = gemstone_py_native.NativeSession.login_from_env()
try:
    assert session.eval_smallint("3 + 4") == 7
finally:
    session.logout()
PY
```

Keep this adapter thin. Python package code can add Pythonic APIs, async
facades, packaging, and backward-compatible return behavior above this native
core.
