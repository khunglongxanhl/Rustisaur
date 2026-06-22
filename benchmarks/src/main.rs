//! Rustisaur benchmarks.

use rustisaur_core::{EngineConfig, RustisaurEngine};

fn main() {
    println!("Rustisaur Benchmarks v{}", env!("CARGO_PKG_VERSION"));

    bench_script_execution();
    bench_json_parsing();
}

fn bench_script_execution() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    let iterations = 1000;
    let start = std::time::Instant::now();

    for i in 0..iterations {
        engine.execute_script(&format!("return {i} + 1")).unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "Script execution: {iterations} iterations in {elapsed:?} ({:.2} ops/sec)",
        iterations as f64 / elapsed.as_secs_f64()
    );
}

fn bench_json_parsing() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    let iterations = 500;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        engine
            .execute_script(
                r#"return rex.json.stringify(rex.json.parse('{"a":1,"b":[1,2,3],"c":"test"}'))"#,
            )
            .unwrap();
    }

    let elapsed = start.elapsed();
    println!(
        "JSON parse/stringify: {iterations} iterations in {elapsed:?} ({:.2} ops/sec)",
        iterations as f64 / elapsed.as_secs_f64()
    );
}
