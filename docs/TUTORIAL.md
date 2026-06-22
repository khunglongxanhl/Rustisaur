# Rustisaur Tutorial

## Installation

1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Clone and build Rustisaur:

```bash
git clone https://github.com/Rustisaur/Rustisaur.git
cd Rustisaur
cargo build --release
```

## Your First Script

Create `hello.rex`:

```lua
rex.print("Hello, Rustisaur!")
```

Run it:

```bash
cargo run --release -p Rustisaur-cli -- run hello.rex
```

## Using the REPL

```bash
cargo run --release -p Rustisaur-cli -- repl
```

```
rex> rex.print("Interactive mode!")
rex> return 2 + 2
=> 4
rex> .exit
```

## Working with Files

```lua
-- Write and read files
rex.fs.write("greeting.txt", "Hello, file system!")
local content = rex.fs.read("greeting.txt")
rex.print(content)

-- Append to files
rex.fs.append("log.txt", os.date() .. " - Application started\n")
```

## HTTP Requests

```lua
local response = rex.http.get("https://httpbin.org/get")
rex.print("Status: " .. response.status)

local data = rex.json.parse(response.body)
rex.print("Origin: " .. (data.origin or "unknown"))
```

## Error Handling

```lua
local success, result = pcall(function()
    return rex.json.parse("{invalid json")
end)

if not success then
    rex.print.error("Parse failed: " .. result)
end
```

## Embedding in Rust

```rust
use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    engine.execute_file("hello.rex".as_ref()).unwrap();
}
```

## Next Steps

- Read the [Lua API Reference](LUA_API.md)
- Explore [examples](../examples/)
- Review [Security Guidelines](SECURITY.md)
