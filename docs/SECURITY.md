# Security Guidelines

## Sandbox Mode

Enable sandbox mode for untrusted scripts:

```rust
use rustisaur_core::{EngineConfig, RustisaurEngine};

let engine = RustisaurEngine::new(EngineConfig::sandboxed()).unwrap();
```

### Disabled Functions

- `os.execute` — shell command execution
- `os.exit` — process termination
- `io.popen` — pipe to shell
- `loadfile` / `dofile` / `load` — dynamic code loading

### Resource Limits

- **Memory**: 128 MB default (configurable)
- **Instructions**: 1,000,000 default (configurable via hook)

## Best Practices

1. Always sandbox untrusted user scripts
2. Validate all file paths before I/O operations
3. Never expose raw Rust pointers to Lua
4. Use HTTPS for all network requests in production
5. Keep Rustisaur updated for security patches
6. Run scripts with minimal OS permissions
7. Log script execution for audit trails

## Threat Model

Rustisaur sandbox mode protects against:
- Arbitrary shell execution
- Unbounded memory consumption
- Infinite loops (instruction limit)

Rustisaur does NOT protect against:
- Network exfiltration (HTTP is allowed by default)
- File system access within allowed paths
- CPU exhaustion (partial — instruction limit helps)

Configure additional restrictions based on your deployment needs.
