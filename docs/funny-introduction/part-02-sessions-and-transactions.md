# Part 2: Sessions and Transactions

A GemStone session is not just a socket with better manners. It is logged in,
transactional, and attached to server-side state. Treat it casually and your
code will eventually ask why a write "worked" and then vanished.

`gemstone-rs` makes the session visible:

```rust
let mut session = Session::login(Config::from_env()?)?;
println!("session id: {}", session.session_id());
session.logout()?;
```

`Session` also logs out on drop. Explicit `logout()` is still useful when a
command-line example wants to show the full lifecycle.

## The Transaction Wrapper

```rust
session.transaction(|session| {
    let value = session.new_string("committed")?;
    session.global_put("GemStoneRsCommitted", value)
})?;
```

This wrapper is intentionally boring:

- `Ok` means commit
- `Err` means abort
- the error is returned to the caller

Boring transaction helpers are good. Exciting transaction helpers are usually a
future incident report.

## Manual Control

Sometimes you want the raw controls:

```rust
let needs_commit = session.needs_commit()?;
let in_transaction = session.in_transaction()?;
session.commit()?;
session.abort()?;
```

Manual control is useful for maintenance tools and diagnostics. Application
code should usually prefer the transaction wrapper.

## Run the Example

```bash
cargo run -p gemstone-rs --example transactions
```

It writes one committed value and then intentionally takes the abort path for a
second value. The point is not drama. The point is knowing which writes survive.
