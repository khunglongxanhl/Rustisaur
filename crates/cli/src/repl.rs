//! Interactive REPL for Rustisaur.

use anyhow::{Context, Result};
use mlua::Value;
use rustisaur_core::{EngineConfig, RustisaurEngine};
use rustyline::DefaultEditor;

pub struct RexREPL {
    rl: DefaultEditor,
    engine: RustisaurEngine,
}

impl RexREPL {
    pub fn new() -> Result<Self> {
        let rl = DefaultEditor::new().context("Failed to create line editor")?;
        let engine = RustisaurEngine::new(EngineConfig::default())
            .context("Failed to initialize Rustisaur engine")?;

        Ok(Self { rl, engine })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Rustisaur REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type '.exit' to quit, '.help' for help\n");

        loop {
            let readline = self.rl.readline("rex> ");

            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    let _ = self.rl.add_history_entry(trimmed);

                    if trimmed == ".exit" || trimmed == ".quit" {
                        break;
                    } else if trimmed == ".help" {
                        self.print_help();
                    } else if trimmed == ".clear" {
                        print!("\x1B[2J\x1B[1;1H");
                    } else if trimmed.starts_with(".load ") {
                        let file = trimmed[6..].trim();
                        match self.engine.execute_file(file.as_ref()) {
                            Ok(result) => self.print_result(result),
                            Err(e) => eprintln!("Error: {e}"),
                        }
                    } else {
                        match self.engine.execute_script(trimmed) {
                            Ok(result) => self.print_result(result),
                            Err(e) => eprintln!("Error: {e}"),
                        }
                    }
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    fn print_help(&self) {
        println!("Commands:");
        println!("  .exit          - Exit the REPL");
        println!("  .quit          - Exit the REPL");
        println!("  .help          - Show this help");
        println!("  .load <file>   - Load and execute a file");
        println!("  .clear         - Clear the screen");
        println!();
        println!("Rustisaur API is available via the `rex` global table.");
        println!("Example: rex.print('Hello, Rustisaur!')");
    }

    fn print_result(&self, value: Value) {
        match value {
            Value::Nil => {}
            Value::Boolean(b) => println!("=> {b}"),
            Value::Integer(i) => println!("=> {i}"),
            Value::Number(n) => println!("=> {n}"),
            Value::String(s) => println!("=> {}", s.to_str().unwrap_or("")),
            other => println!("=> {other:?}"),
        }
    }
}
