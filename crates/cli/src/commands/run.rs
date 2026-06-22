//! Run command - execute a script file.

use std::path::Path;

use anyhow::{Context, Result};
use mlua::Value;
use rustisaur_core::{EngineConfig, RustisaurEngine};

pub fn execute(file: &str) -> Result<()> {
    let path = Path::new(file);
    let engine = RustisaurEngine::new(EngineConfig::default())
        .context("Failed to initialize Rustisaur engine")?;

    let result = engine
        .execute_file(path)
        .with_context(|| format!("Failed to execute script: {file}"))?;

    print_result(result);
    Ok(())
}

fn print_result(value: Value) {
    match value {
        Value::Nil => {}
        Value::Boolean(b) => println!("{b}"),
        Value::Integer(i) => println!("{i}"),
        Value::Number(n) => println!("{n}"),
        Value::String(s) => println!("{}", s.to_str().unwrap_or("")),
        other => println!("{other:?}"),
    }
}
