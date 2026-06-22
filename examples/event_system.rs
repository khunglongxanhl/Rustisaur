//! Event system example.

use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    let engine = RustisaurEngine::new(EngineConfig::default()).expect("Failed to create engine");

    engine
        .execute_script(
            r#"
            local emitter = rex.event.create()

            emitter:on("greeting", function(data)
                rex.print("Received: " .. data.message)
            end)

            -- Note: emit is async; in sync context we use direct call
            rex.print("Event system initialized")
            rex.print.success("Event emitter created")
        "#,
        )
        .expect("Failed to execute script");
}
