# Contributing to Rustisaur

Thank you for your interest in contributing to Rustisaur!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/Rustisaur.git`
3. Build the project: `cargo build`
4. Run tests: `cargo test --workspace`

## Development Setup

```bash
cd Rustisaur
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

## Code Standards

- Follow Rust idioms and clippy recommendations
- Use `Result<T, E>` everywhere in library code (no `unwrap`/`expect`)
- Document all public APIs with rustdoc
- Write tests for new functionality
- Match existing code style and conventions

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes with tests
3. Ensure `cargo test` and `cargo clippy` pass
4. Submit a pull request with a clear description

## Reporting Issues

Please include:
- Rustisaur version (`Rustisaur version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
