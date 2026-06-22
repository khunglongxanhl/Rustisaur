# Rustisaur

**Rustisaur** is a high-performance embedded scripting language engine that combines the strengths of Rust, Lua, and C. It provides memory safety without garbage collection overhead, async I/O via Tokio, and a lightweight Lua-based scripting layer.

## Features

- **Rust Foundation** — Zero-cost abstractions, memory safety, single binary deployment
- **Lua Scripting Layer** — Dynamic typing, hot-reloadable scripts, easy-to-learn syntax
- **Async I/O** — Full Tokio integration for files, HTTP, WebSockets, and more
- **Security Sandbox** — Resource limits and disabled dangerous functions for untrusted scripts
- **Rich Standard Library** — File I/O, HTTP client, JSON, events, process utilities
- **Cross-Platform** — Linux, macOS, and Windows support

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- C compiler (for vendored Lua via mlua)

### Build

```bash
cd Rustisaur
cargo build --release
```

### Run the CLI

```bash
# Run a Rustisaur file
cargo run --release -p Rustisaur-cli -- run examples/rustisaur_examples/hello.rex

# Start the interactive REPL
cargo run --release -p Rustisaur-cli -- repl

# Check script syntax
cargo run --release -p Rustisaur-cli -- check examples/rustisaur_examples/hello.rex

# Show version
cargo run --release -p Rustisaur-cli -- version
```

### Hello World

Create `hello.rex`:

```lua
rex.print("Hello, Rustisaur!")
rex.print.success("It works!")
```

Run it:

```bash
cargo run --release -p Rustisaur-cli -- run hello.rex
```

## Rustisaur API

Rustisaur scripts use Lua 5.4 syntax with the `rex` global table:

```lua
-- Console output
rex.print("Hello, World!")
rex.print.error("Something went wrong")

-- File I/O
local content = rex.fs.read("file.txt")
rex.fs.write("output.txt", "Hello")

-- HTTP (async)
local response = rex.http.get("https://api.example.com/data")
local data = rex.json.parse(response.body)

-- JSON
local obj = rex.json.parse('{"name": "John"}')
local str = rex.json.stringify({name = "Jane"})

-- Environment
local path = rex.env.get("PATH")
local cwd = rex.process.cwd()

-- Math & strings
rex.print(rex.math.random(1, 100))
rex.print(rex.string.upper("hello"))
```

See [docs/LUA_API.md](docs/LUA_API.md) for the complete API reference.

## Project Structure

```
Rustisaur/
├── crates/
│   ├── core/           # Rustisaur engine
│   ├── lua-bridge/     # Lua integration & sandbox
│   ├── stdlib/         # Standard library & Lua bindings
│   ├── event-system/   # Event-driven architecture
│   ├── plugin-system/  # Plugin architecture
│   ├── cli/            # Command-line interface
│   └── ffi-layer/      # C FFI bindings
├── examples/           # Rust and .rex examples
├── tests/              # Integration tests
├── docs/               # Documentation
└── benchmarks/         # Performance benchmarks
```

## Examples

```bash
# Rust examples
cargo run --release -p Rustisaur-core --example basic_hello
cargo run --release -p Rustisaur-core --example file_io

# Rustisaur examples
cargo run --release -p Rustisaur-cli -- run examples/rustisaur_examples/hello.rex
cargo run --release -p Rustisaur-cli -- run examples/rustisaur_examples/file_ops.rex
```

## Testing

```bash
cargo test --workspace
cargo test -p Rustisaur-tests
```

## Benchmarks

```bash
cargo run --release -p Rustisaur-benchmarks
```

## Embedding Rustisaur in Rust

```rust
use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    engine.execute_script(r#"rex.print("Hello from embedded Rustisaur!")"#).unwrap();
}
```

## Security

Enable sandbox mode for untrusted scripts:

```rust
let engine = RustisaurEngine::new(EngineConfig::sandboxed()).unwrap();
```

Sandbox mode disables `os.execute`, `io.popen`, `loadfile`, and enforces memory/instruction limits.

## License

MIT — see [LICENSE](LICENSE)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
