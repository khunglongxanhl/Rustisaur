use mlua::{Lua, Result, Table, Value, Function};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Quản lý việc load modules theo yêu cầu
pub struct LazyLoader {
    /// Cache các modules đã load
    loaded_modules: Arc<Mutex<HashMap<String, Table>>>,
    
    /// Factory functions để tạo modules
    factories: HashMap<String, Box<dyn Fn(&Lua) -> Result<Table> + Send + Sync>>,
}

impl LazyLoader {
    pub fn new() -> Self {
        Self {
            loaded_modules: Arc::new(Mutex::new(HashMap::new())),
            factories: HashMap::new(),
        }
    }
    
    /// Đăng ký một module factory
    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn(&Lua) -> Result<Table> + Send + Sync + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }
    
    /// Tạo proxy table với metatable __index
    pub fn create_proxy(&self, lua: &Lua) -> Result<Table> {
        let proxy = lua.create_table()?;
        let loaded = self.loaded_modules.clone();
        let factories = Arc::new(self.factories.clone());
        
        // Tạo metatable với __index
        let metatable = lua.create_table()?;
        
        let index_func = lua.create_function(move |lua, (table, key): (Table, String)| {
            // Kiểm tra cache trước
            {
                let cache = loaded.lock().unwrap();
                if let Some(module) = cache.get(&key) {
                    return Ok(Value::Table(module.clone()));
                }
            }
            
            // Không có trong cache, load từ factory
            let factories = factories.clone();
            if let Some(factory) = factories.get(&key) {
                println!("🔄 Lazy loading module: {}", key);
                let module = factory(lua)?;
                
                // Lưu vào cache
                let mut cache = loaded.lock().unwrap();
                cache.insert(key, module.clone());
                
                Ok(Value::Table(module))
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Module '{}' not found", key
                )))
            }
        })?;
        
        metatable.set("__index", index_func)?;
        proxy.set_metatable(Some(metatable));
        
        Ok(proxy)
    }
}