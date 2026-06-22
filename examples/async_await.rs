//! Async/await example.

use rustisaur_core::{EngineConfig, RustisaurEngine};

#[tokio::main]
async fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).expect("Failed to create engine");

    engine
        .execute_script(
            r#"
            local content = rex.fs.read_async("Cargo.toml")
            rex.print("Read Cargo.toml asynchronously")
            rex.print("First 100 chars: " .. content:sub(1, 100))
        "#,
        )
        .expect("Failed to execute script");
}
