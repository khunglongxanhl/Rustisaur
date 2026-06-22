//! Rustisaur integration tests.

use rustisaur_core::{EngineConfig, RustisaurEngine};

#[test]
fn hello_world_script() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    let result = engine
        .execute_script("rex.print('Hello'); return 42")
        .unwrap();
    match result {
        mlua::Value::Integer(i) => assert_eq!(i, 42),
        _ => panic!("Expected integer 42"),
    }
}

#[test]
fn json_parse_and_stringify() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    engine
        .execute_script(
            r#"
            local obj = rex.json.parse('{"name": "test", "value": 123}')
            assert(obj.name == "test")
            assert(obj.value == 123)
            local s = rex.json.stringify(obj)
            assert(type(s) == "string")
        "#,
        )
        .unwrap();
}

#[test]
fn file_read_write() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    engine
        .execute_script(
            r#"
            rex.fs.write("_test_rex.txt", "test content")
            local c = rex.fs.read("_test_rex.txt")
            assert(c == "test content")
            os.remove("_test_rex.txt")
        "#,
        )
        .unwrap();
}

#[test]
fn math_and_string_utils() {
    let engine = RustisaurEngine::new(EngineConfig::default()).unwrap();
    engine
        .execute_script(
            r#"
            assert(rex.string.upper("hello") == "HELLO")
            assert(rex.string.lower("WORLD") == "world")
            assert(rex.math.max(1, 5, 3) == 5)
            assert(rex.math.min(1, 5, 3) == 1)
        "#,
        )
        .unwrap();
}

#[test]
fn sandbox_mode_blocks_os_execute() {
    let engine = RustisaurEngine::new(EngineConfig::sandboxed()).unwrap();
    assert!(engine.execute_script("os.execute('echo test')").is_err());
}
