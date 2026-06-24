<div align="center">
  <img src="rustisaur-vscode/icons/Logo-Rustisaur.png" alt="Rustisaur Logo" width="200"/>
  
  # 🦖 Rustisaur
  
  **A high-performance embedded scripting language engine**
  
  *Combining the power of Rust with the simplicity of Lua*
  
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/khunglongxanhl/Rustisaur)
  [![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
  [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
  [![Lua](https://img.shields.io/badge/lua-5.4-yellow)](https://www.lua.org/)
  [![VS Code](https://img.shields.io/badge/VS%20Code-Extension-blueviolet)](https://github.com/khunglongxanhl/Rustisaur/releases)
  
  [Features](#-features) • [Quick Start](#-quick-start) • [API](#-rustisaur-api) • [Performance](#-performance) • [VS Code Extension](#-vs-code-extension)
</div>

---

## ✨ Features

### 🚀 Core Capabilities
- **⚡ Lazy Loading** — 55x faster startup (0.2ms vs 11ms), modules loaded on-demand
- **💾 Memory Efficient** — 5x less RAM usage (1MB vs 5MB for basic scripts)
- **🔒 Security Sandbox** — Resource limits and disabled dangerous functions
- **🔄 Hot Reload** — Reload scripts without restarting the engine
- **📦 65+ Built-in Functions** — String, File, JSON, Math, OS, Table operations
- **🌐 Async I/O** — Full Tokio integration for files, HTTP, WebSockets
- **🎯 Cross-Platform** — Linux, macOS, and Windows support

### 🛠️ Technical Highlights
- **Rust Foundation** — Zero-cost abstractions, memory safety, single binary
- **Lua 5.4 Scripting** — Dynamic typing, easy-to-learn syntax
- **Script Caching** — Automatic bytecode caching for 10-50x faster execution
- **Engine Pooling** — Reuse engines for high-performance scenarios
- **Event System** — Built-in event-driven architecture
- **Plugin System** — Extensible via dynamic plugins
- **FFI Layer** — C bindings for integration with other languages

---

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- C compiler (for vendored Lua via mlua)

### Build

```bash
git clone https://github.com/khunglongxanhl/Rustisaur.git
cd Rustisaur
cargo build --release
Run the CLI
# Run a Rustisaur script
cargo run --release --bin rustisaur -- run examples/hello.rex

# Start interactive REPL
cargo run --release --bin rustisaur -- repl

# Check script syntax
cargo run --release --bin rustisaur -- check examples/hello.rex

# Show version
cargo run --release --bin rustisaur -- version

Hello World
Create hello.rex:
-- Simple Hello World
rex.print("Hello, Rustisaur!")

-- String operations
local name = "  world  "
rex.print("Hello, " .. rex.string.trim(name) .. "!")

-- Math operations
rex.print("PI = " .. rex.math.pi())
rex.print("sqrt(16) = " .. rex.math.sqrt(16))

-- File operations
rex.fs.write("test.txt", "Hello from Rustisaur!")
local content = rex.fs.read("test.txt")
rex.print("File content: " .. content)
rex.fs.delete("test.txt")

Run it:
cargo run --release --bin rustisaur -- run hello.rex

📚 Rustisaur API
Rustisaur scripts use Lua 5.4 syntax with the rex global table:

🖨️ Console Output
rex.print("Hello, World!")
rex.print("User input: " .. rex.input("Enter name: "))
📁 File System (14 functions)
-- Read/Write
local content = rex.fs.read("file.txt")
rex.fs.write("output.txt", "Hello")

-- File operations
if rex.fs.exists("data.json") then
    local meta = rex.fs.metadata("data.json")
    rex.print("Size: " .. meta.size .. " bytes")
end

-- Directory operations
rex.fs.mkdir("new_folder")
local files = rex.fs.list("./scripts")
for i, file in ipairs(files) do
    rex.print(i .. ": " .. file)
end
📝 String Operations (20 functions)
-- Transformations
rex.print(rex.string.upper("hello"))      -- HELLO
rex.print(rex.string.capitalize("world")) -- World
rex.print(rex.string.reverse("rust"))     -- tsur

-- Split & Join
local parts = rex.string.split("a,b,c", ",")
local joined = rex.string.join(parts, "-")  -- a-b-c

-- Check
rex.print(rex.string.starts_with("hello", "hel"))  -- true
rex.print(rex.string.contains("world", "orl"))     -- true
🔢 Math Operations (12 functions)
rex.print(rex.math.max(3, 7, 1, 9, 2))  -- 9
rex.print(rex.math.min(3, 7, 1, 9, 2))  -- 1
rex.print(rex.math.round(3.7))          -- 4
rex.print(rex.math.sqrt(16))            -- 4
rex.print(rex.math.random(1, 100))      -- Random 1-100
📊 Table Operations (10 functions)
local t = {5, 2, 8, 1, 9}
local sorted = rex.table.sort(t)
local reversed = rex.table.reverse(sorted)
local unique = rex.table.unique({1, 2, 2, 3, 3})

-- Filter and map
local evens = rex.table.filter(t, function(n) return n % 2 == 0 end)
local doubled = rex.table.map(t, function(n) return n * 2 end)
📦 JSON Operations
-- Parse JSON
local data = rex.json.parse('{"name": "John", "age": 30}')
rex.print(data.name)  -- John

-- Stringify to JSON
local obj = {name = "Jane", age = 25}
local json_str = rex.json.stringify(obj)
⚙️ OS Operations (5 functions)
rex.print("Time: " .. rex.os.time())
rex.print("CWD: " .. rex.os.cwd())
rex.print("PATH: " .. rex.os.env("PATH"))

rex.os.sleep(1000)  -- Sleep 1 second
See docs/LUA_API.md for the complete API reference.
🔒 Security
Enable sandbox mode for untrusted scripts:
let config = EngineConfig::sandboxed();
let engine = RustisaurEngine::new(config).unwrap();

// Sandbox mode:
// - Disables os.execute, io.popen, loadfile, dofile
// - Enforces memory limits (default: 128MB)
// - Enforces time limits (default: 30s)
// - Validates scripts for dangerous patterns
Security Features
✅ Dangerous Pattern Detection — Blocks os.execute, io.popen, etc.
✅ Resource Limits — Memory, time, instruction count
✅ File System Restrictions — Whitelist allowed directories
✅ Network Restrictions — Control network access
✅ Module Restrictions — Disable specific modules
📦 Installation
From Source
git clone https://github.com/khunglongxanhl/Rustisaur.git
cd Rustisaur
cargo build --release
Using Docker
# Build image
docker build -t rustisaur .

# Run script
docker run --rm -v $(pwd):/app rustisaur run /app/script.rex

# Interactive REPL
docker run --rm -it rustisaur repl
🤝 Contributing
Contributions are welcome! Please see CONTRIBUTING.md for guidelines.
👤 Author
khunglongxanhl
GitHub: @khunglongxanhl
Repository: Rustisaur

🌟 Show Your Support
If you find Rustisaur useful, consider giving it a ⭐ on GitHub!
<div align="center">
<sub>Built with ❤️ by <a href="https://github.com/khunglongxanhl">khunglongxanhl</a></sub>
<br/>
<sub>🦖 Rustisaur - Where Rust meets Lua</sub>
</div>
```