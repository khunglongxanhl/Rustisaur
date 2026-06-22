# Rustisaur Rust API Reference

## Core Engine

```rust
use rustisaur_core::{EngineConfig, RustisaurEngine};

// Create engine
let engine = RustisaurEngine::new(EngineConfig::default())?;

// Execute script string
let result = engine.execute_script("return 42")?;

// Execute script file
let result = engine.execute_file("script.rex".as_ref())?;

// Register custom function
engine.register_function("my_func", |lua, arg| {
    Ok(mlua::Value::String(lua.create_string("result")?))
})?;

// Shutdown
engine.shutdown()?;
```

## Configuration

```rust
use rustisaur_core::EngineConfig;
use tracing::Level;

let config = EngineConfig {
    max_memory_mb: 256,
    script_timeout_secs: 60,
    enable_async: true,
    sandbox_mode: true,
    log_level: Level::DEBUG,
};
```

## Error Handling

```rust
use rustisaur_core::{RexError, EngineError, Result};

fn run() -> Result<()> {
    // Errors propagate via Result types
    Ok(())
}
```

## Event System

```rust
use rustisaur_event_system::EventEmitter;
use serde_json::json;

let mut emitter = EventEmitter::new();
let mut rx = emitter.on("event");
emitter.emit_sync("event", json!({"key": "value"}))?;
```

## Plugin System

```rust
use rustisaur_plugin_system::PluginManager;

let mut manager = PluginManager::new();
manager.load_and_register("my-plugin", "0.1.0")?;
manager.shutdown_all()?;
```

## FFI (C)

```c
#include "Rustisaur.h"

RustisaurHandle* engine = rustisaur_create();
char* result = rustisaur_execute(engine, "rex.print('hello')");
rustisaur_free_string(result);
rustisaur_destroy(engine);
```
