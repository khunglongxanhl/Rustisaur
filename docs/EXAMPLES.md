# Examples Guide

## Rustisaur Examples (.rex)

Located in `examples/rustisaur_examples/`:

| File | Description |
|------|-------------|
| `hello.rex` | Hello World with user input |
| `file_ops.rex` | File read/write/append |
| `http_request.rex` | HTTP GET with JSON parsing |
| `async_example.rex` | Async file reading |

```bash
cargo run --release -p Rustisaur-cli -- run examples/rustisaur_examples/hello.rex
```

## Rust Examples

Located in `examples/`:

| File | Description |
|------|-------------|
| `basic_hello.rs` | Minimal embedded Rustisaur |
| `file_io.rs` | File I/O from Rust-hosted scripts |
| `http_client.rs` | HTTP requests (requires network) |
| `custom_functions.rs` | Register Rust functions in Lua |
| `async_await.rs` | Async file operations |
| `event_system.rs` | Event emitter demo |

```bash
cargo run --release -p Rustisaur-core --example basic_hello
cargo run --release -p Rustisaur-core --example file_io
```

## REPL Examples

```bash
cargo run --release -p Rustisaur-cli -- repl
```

```
rex> rex.print("Hello!")
rex> return rex.math.max(10, 20, 5)
=> 20
rex> .load examples/rustisaur_examples/hello.rex
```

## Embedding Example

```rust
use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    engine.execute_script(r#"rex.print("Embedded Rustisaur!")"#).unwrap();
}
```
