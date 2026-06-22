# Rustisaur Architecture

## Overview

Rustisaur is a layered scripting engine:

```
┌─────────────────────────────────────┐
│           Rustisaur CLI             │
├─────────────────────────────────────┤
│         Rustisaur Core              │
│    (Engine, Runtime, Config)        │
├─────────────────────────────────────┤
│         Lua Bridge                  │
│  (State, Sandbox, Async, Bindings)  │
├─────────────────────────────────────┤
│         Standard Library            │
│  (I/O, Net, Data, Sys, Utils)       │
├─────────────────────────────────────┤
│    Event System │ Plugin System     │
├─────────────────────────────────────┤
│         FFI Layer (C)               │
├─────────────────────────────────────┤
│    Tokio │ mlua (Lua 5.4) │ Rust    │
└─────────────────────────────────────┘
```

## Crates

| Crate | Purpose |
|-------|---------|
| `Rustisaur-core` | Main engine, runtime management, configuration |
| `Rustisaur-lua-bridge` | Lua state, sandbox, type conversions |
| `Rustisaur-stdlib` | Standard library Rust modules + Lua bindings |
| `Rustisaur-event-system` | Broadcast-based event emitters |
| `Rustisaur-plugin-system` | Plugin loading and registry |
| `Rustisaur-cli` | Command-line interface |
| `Rustisaur-ffi-layer` | C FFI for embedding |

## Script Execution Flow

1. CLI or embedder creates `RustisaurEngine` with `EngineConfig`
2. Engine initializes Tokio runtime and Lua state via `LuaStateManager`
3. Standard library registers the `rex` global table
4. Optional sandbox restrictions are applied
5. Script source is loaded and executed via mlua
6. Results propagate back through the error handling chain

## Memory Model

- No garbage collector in Rust layer
- Lua uses its own incremental GC (standard Lua 5.4)
- Sandbox enforces memory limits via mlua's `set_memory_limit`
- Instruction counting hook prevents infinite loops

## Async Model

- Tokio provides the async runtime
- mlua async functions expose async I/O to Lua scripts
- HTTP and file async operations use `create_async_function`
