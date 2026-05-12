# CLI Browser Walkthrough

This walkthrough proves the read-only browser path from the terminal. It
requires live GemStone credentials in the environment.

```bash
gemstone-rs doctor --live
gemstone-rs browse dictionaries
gemstone-rs browse classes UserGlobals
gemstone-rs browse protocols Object
gemstone-rs browse methods Object "-- all --"
gemstone-rs browse source Object printString
```

Expected output:

```text
doctor live probe: ok
UserGlobals
--
printing
printString
```

Use explicit dictionary-qualified class references in codegen configs when a
class is not resolvable through the active user's symbol list:

```text
class = UserGlobals:OkzBooking
method = UserGlobals:OkzBooking>>findById: | args=id | return=Oop
```

The same browser API feeds the local explorer and the VS Code sidebar.
