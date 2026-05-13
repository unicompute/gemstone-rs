# gemstone-gci

`gemstone-gci` is the low-level Rust GCI layer for GemStone/S. It dynamically
loads `libgcirpc` at runtime, exposes OOP constants and helpers, and wraps the
raw C ABI functions used by higher-level clients.

The crate does not require GemStone headers at build time. Set one of these at
runtime:

```bash
export GS_LIB_PATH=/path/to/libgcirpc.dylib
export GS_LIB=/path/to/gemstone/lib
export GEMSTONE=/path/to/gemstone
```

Use `resolve_library_path_with_source` when diagnostics need to show whether
`libgcirpc` came from an explicit path, `GS_LIB_PATH`, `GS_LIB`, or
`GEMSTONE/lib`, and which path or directory was searched.

Most applications should use the safe `gemstone` crate instead of calling this
crate directly.
