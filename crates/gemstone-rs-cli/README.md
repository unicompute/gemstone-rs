# gemstone-rs-cli

Command line tools for `gemstone-rs`.

The binary name is `gemstone-rs`:

```bash
cargo run -p gemstone-rs-cli -- eval "3 + 4"
cargo run -p gemstone-rs-cli -- browse classes
cargo run -p gemstone-rs-cli -- inspect oop 20
cargo run -p gemstone-rs-cli -- codegen check
cargo run -p gemstone-rs-cli -- codegen generate
```

The CLI uses the same GemStone environment variables as the library:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```
