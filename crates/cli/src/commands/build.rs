//! Build command - bundle scripts (placeholder).

use anyhow::{bail, Result};

pub fn execute(file: &str, output: Option<&str>) -> Result<()> {
    let out = output.unwrap_or("bundle.rex");
    println!("Building {file} -> {out}");
    bail!("Build command is not yet implemented in v0.1.0")
}
