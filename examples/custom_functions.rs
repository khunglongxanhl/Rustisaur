//! Custom Rust functions in Lua example.

use mlua::Value;
use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    let mut engine =
        RustisaurEngine::new(EngineConfig::default()).expect("Failed to create engine");

    engine
        .register_function("greet", |lua, arg: Value| {
            let name = match arg {
                Value::String(s) => s.to_str()?.to_string(),
                _ => "World".to_string(),
            };
            Ok(Value::String(lua.create_string(format!("Hello, {name}!"))?))
        })
        .expect("Failed to register function");

    engine
        .execute_script(
            r#"
            local msg = greet("Rustisaur")
            rex.print(msg)
        "#,
        )
        .expect("Failed to execute script");
}
