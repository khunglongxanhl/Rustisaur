//! HTTP client example (requires network).

use rustisaur_core::{EngineConfig, RustisaurEngine};

#[tokio::main]
async fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).expect("Failed to create engine");

    let script = r#"
        local response = rex.http.get("https://httpbin.org/get")
        rex.print("Status: " .. response.status)
        rex.print("Body length: " .. #response.body)
    "#;

    engine
        .execute_script(script)
        .expect("Failed to execute script");
}
