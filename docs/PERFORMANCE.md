# Performance Guide

## Running Benchmarks

```bash
cargo run --release -p Rustisaur-benchmarks
```

## Optimization Tips

### Script Execution

- Pre-compile frequently-run scripts by keeping the engine alive
- Reuse a single `RustisaurEngine` instance across executions
- Use sandbox mode only when needed (hooks add overhead)

### I/O

- Use async file operations (`rex.fs.read_async`) for large files
- Batch HTTP requests rather than sequential calls
- Prefer JSON parsing over manual string manipulation

### Memory

- Configure `max_memory_mb` appropriately for your workload
- Avoid creating large Lua tables in hot paths
- Use streaming for large file processing (future API)

## Architecture Advantages

- **No GC in Rust layer** — Predictable memory usage for host application
- **Vendored Lua** — Single binary, no external Lua dependency
- **Tokio async** — Efficient I/O without blocking threads
- **Zero-copy where possible** — String handling via mlua's efficient conversions

## Expected Performance

Initial v0.1.0 targets (release build):
- Script execution: >10,000 simple evals/sec
- JSON parse/stringify: >5,000 ops/sec
- File read (1KB): >50,000 ops/sec

Run benchmarks on your hardware for accurate numbers.
