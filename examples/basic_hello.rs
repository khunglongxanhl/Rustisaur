//! Basic Hello World example.

use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).expect("Failed to create engine");

    engine
        .execute_script(
            r#"
            rex.print("Hello, Rustisaur!")
            rex.print.success("Engine initialized successfully")
        "#,
        )
        .expect("Failed to execute script");
}
