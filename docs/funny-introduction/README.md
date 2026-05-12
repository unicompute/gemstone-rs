# A Practical but Lighter Introduction to gemstone-rs

This is the guided version of the docs. It still shows real APIs, real command
lines, and real failure modes, but it lets the explanation breathe a little.

Read in order:

1. [Why gemstone-rs Exists](part-01-why-gemstone-rs-exists.md)
2. [Sessions and Transactions](part-02-sessions-and-transactions.md)
3. [OOPs, Values, and the Browser](part-03-oops-values-and-browser.md)
4. [Codegen, Explorer, and VS Code](part-04-codegen-explorer-and-vscode.md)

The short version:

```bash
cargo add gemstone-rs
cargo install gemstone-rs-cli
gemstone-rs eval "3 + 4"
```

If that prints `7`, Rust and GemStone/S have successfully shaken hands.
