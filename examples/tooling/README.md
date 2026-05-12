# gemstone-rs Tooling Examples

These examples prove the tooling around the Rust API:

| Tooling surface | Example |
| --- | --- |
| Codegen workflow | `cargo run -p gemstone-rs --example codegen_workflow` |
| Local explorer | [explorer.md](explorer.md) |
| VS Code Workbench | [vscode-workbench.md](vscode-workbench.md) |

The tooling intentionally calls the same API and CLI surfaces as application
code. If one of these examples fails, debug the underlying CLI/API command
first.

## Common Environment

Live tooling expects:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_STONE_NAME=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

Offline codegen examples do not require a live stone.
