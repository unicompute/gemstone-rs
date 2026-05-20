# gemstone-rs Documentation

This directory contains the human-facing guides for `gemstone-rs`.

## Start Here

| Guide | Use it when |
| --- | --- |
| [Setup Guide](setup-guide.md) | You need Rust, GemStone environment variables, install commands, and first login checks. |
| [Examples Guide](examples-guide.md) | You want to know which example to run for each feature. |
| [Feature Map](feature-map.md) | You want the Rust equivalent of `gemstone-examples plan3-map`, with crates, examples, docs, and gemstone-py references. |
| [User Manual](user-manual.md) | You want the main Rust API surface in one place. |
| [Cookbook](cookbook.md) | You want task-focused recipes for sessions, transactions, browser calls, codegen, explorer, and VS Code. |
| [gemstone-py vs gemstone-rs](gemstone-py-vs-gemstone-rs.md) | You want install paths, use cases, maturity, and feature differences between the Python and Rust projects. |
| [gemstone-js vs gemstone-py](gemstone-js-vs-gemstone-py.md) | You want the TypeScript/Python comparison, maturity matrix, and gemstone-js catch-up batches. |
| [BridgeRoot and Object Mapping](object-mapping.md) | You want BridgeRoot storage, dictionary mapping, `Remote<T>` object handles, materialization profiles, and explicit Rust-to-GemStone value mapping. |
| [Codegen Guide](codegen.md) | You want generated Rust wrappers for GemStone classes and methods. |
| [Codegen Profile Schema](profile-schema.md) | You want project profile JSON validation for explorer and VS Code workflows. |
| [Explorer Guide](explorer.md) | You want the local HTTP explorer API and safety defaults. |
| [VS Code Workbench](vscode-workbench.md) | You want sidebar browsing, codegen preview/diff/generate, and explorer launch commands. |
| [Screenshot Workflow](screenshots.md) | You want to refresh Explorer and Workbench images before docs or Marketplace releases. |
| [Performance and Safety](performance-safety.md) | You want benchmark guidance, GCI loading notes, and threading rules. |
| [Shared Core Integration](shared-core-integration.md) | You want the plan for `gemstone-py-native` to wrap the Rust core. |
| [Medium Article](medium-article.md) | You want an article-style explanation suitable for publishing or sharing. |
| [Funny Introduction](funny-introduction/README.md) | You want a lighter multi-part tour of the same concepts. |
| [Release Checklist](release-checklist.md) | You want the crate, VSIX, GitHub Release, and post-release verification flow. |

## PDFs

Generated PDFs live in [pdf/](pdf/). Rebuild them after Markdown changes:

```bash
python3 docs/build_pdf_docs.py
```

Check docs links, index coverage, release versions, and PDF targets with:

```bash
make docs-link-check
make docs-index-check
python3 scripts/version_check.py
make docs-pdf-check
```

Refresh screenshots before a visual release:

```bash
make screenshots
```

## Shortest Path

```bash
cargo install gemstone-rs-cli
gemstone-rs --help
```

For library use:

```bash
cargo add gemstone-rs
```

For local examples from a checkout:

```bash
cd /path/to/gemstone-rs
cargo run -p gemstone-rs --example quickstart
```

For the local explorer:

```bash
cargo install gemstone-rs-explorer
gemstone-rs-explorer --port 8787
```

For VS Code:

```text
https://marketplace.visualstudio.com/items?itemName=unicompute.gemstone-rs-workbench
```
