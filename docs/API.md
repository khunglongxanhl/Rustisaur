# Rustisaur API Reference

See also:
- [Lua API Reference](LUA_API.md) — Scripting API for `.rex` files
- [Rust API Reference](RUST_API.md) — Embedding Rustisaur in Rust applications
- [Tutorial](TUTORIAL.md) — Getting started guide

## CLI Commands

| Command | Description |
|---------|-------------|
| `Rustisaur run <file>` | Execute a Rustisaur/Lua file |
| `Rustisaur repl` | Start interactive REPL |
| `Rustisaur check <file>` | Validate syntax without executing |
| `Rustisaur version` | Show version information |
| `Rustisaur build <file>` | Bundle script (planned) |

## Global API Summary

All Rustisaur APIs are under the `rex` global table in Lua scripts.

| Module | Functions |
|--------|-----------|
| `rex.print` | Console output with error/success/warn variants |
| `rex.input` | Read user input |
| `rex.fs` | File read/write/append/exists |
| `rex.http` | HTTP GET/POST client |
| `rex.json` | JSON parse/stringify |
| `rex.env` | Environment variables |
| `rex.process` | Process info (cwd, pid) |
| `rex.time` | Date/time utilities |
| `rex.math` | Math functions |
| `rex.string` | String utilities |
| `rex.table` | Table/collection utilities |
| `rex.event` | Event emitter |
