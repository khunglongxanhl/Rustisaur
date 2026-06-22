//! File I/O example.

use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).expect("Failed to create engine");

    engine
        .execute_script(
            r#"
            rex.fs.write("hello.txt", "Hello from Rustisaur!")
            local content = rex.fs.read("hello.txt")
            rex.print("File contents: " .. content)
            rex.fs.append("hello.txt", "\nAppended line")
            rex.print.success("File I/O complete")
        "#,
        )
        .expect("Failed to execute script");
}
