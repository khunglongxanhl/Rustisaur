//! HTTP Client for making requests to external APIs
//!
//! Features:
//! - GET, POST, PUT, DELETE requests (sync)
//! - Custom headers
//! - JSON body support
//! - Timeout control
//! - Response with status code

use mlua::{Lua, Result as LuaResult, Table, Value};
use reqwest::blocking::Client;
use std::time::Duration;
use tracing::{debug, info};

/// HTTP Client wrapper (sync version)
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// Create new HTTP client with default timeout (30s)
    pub fn new() -> Self {
        debug!("🌐 Creating HTTP Client");
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Rustisaur/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Create HTTP client with custom timeout
    pub fn with_timeout(timeout_secs: u64) -> Self {
        debug!("🌐 Creating HTTP Client with timeout: {}s", timeout_secs);
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Make GET request
    pub fn get(&self, url: &str, headers: Option<&Table>) -> LuaResult<(u16, String)> {
        debug!("📡 GET request to: {}", url);

        let mut request = self.client.get(url);

        // Add headers if provided
        if let Some(headers_table) = headers {
            request = Self::add_headers(request, headers_table)?;
        }

        let response = request
            .send()
            .map_err(|e| mlua::Error::RuntimeError(format!("HTTP GET error: {}", e)))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read response: {}", e)))?;

        info!("✅ GET {} - Status: {}", url, status);

        Ok((status, body))
    }

    /// Make POST request with body
    pub fn post(
        &self,
        url: &str,
        body: Option<String>,
        headers: Option<&Table>,
    ) -> LuaResult<(u16, String)> {
        debug!("📡 POST request to: {}", url);

        let mut request = self.client.post(url);

        // Add headers
        if let Some(headers_table) = headers {
            request = Self::add_headers(request, headers_table)?;
        }

        // Add body
        if let Some(req_body) = body {
            // Auto-detect JSON content type if not specified
            if headers.is_none() || !Self::has_content_type(headers.unwrap()) {
                request = request.header("Content-Type", "application/json");
            }
            request = request.body(req_body);
        }

        let response = request
            .send()
            .map_err(|e| mlua::Error::RuntimeError(format!("HTTP POST error: {}", e)))?;

        let status = response.status().as_u16();
        let response_body = response
            .text()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read response: {}", e)))?;

        info!("✅ POST {} - Status: {}", url, status);

        Ok((status, response_body))
    }

    /// Make PUT request
    pub fn put(
        &self,
        url: &str,
        body: Option<String>,
        headers: Option<&Table>,
    ) -> LuaResult<(u16, String)> {
        debug!("📡 PUT request to: {}", url);

        let mut request = self.client.put(url);

        if let Some(headers_table) = headers {
            request = Self::add_headers(request, headers_table)?;
        }

        if let Some(req_body) = body {
            if headers.is_none() || !Self::has_content_type(headers.unwrap()) {
                request = request.header("Content-Type", "application/json");
            }
            request = request.body(req_body);
        }

        let response = request
            .send()
            .map_err(|e| mlua::Error::RuntimeError(format!("HTTP PUT error: {}", e)))?;

        let status = response.status().as_u16();
        let response_body = response
            .text()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read response: {}", e)))?;

        info!("✅ PUT {} - Status: {}", url, status);

        Ok((status, response_body))
    }

    /// Make DELETE request
    pub fn delete(&self, url: &str, headers: Option<&Table>) -> LuaResult<(u16, String)> {
        debug!("🗑️  DELETE request to: {}", url);

        let mut request = self.client.delete(url);

        if let Some(headers_table) = headers {
            request = Self::add_headers(request, headers_table)?;
        }

        let response = request
            .send()
            .map_err(|e| mlua::Error::RuntimeError(format!("HTTP DELETE error: {}", e)))?;

        let status = response.status().as_u16();
        let response_body = response
            .text()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read response: {}", e)))?;

        info!("✅ DELETE {} - Status: {}", url, status);

        Ok((status, response_body))
    }

    /// Add headers to request
    fn add_headers(
        request: reqwest::blocking::RequestBuilder,
        headers: &Table,
    ) -> LuaResult<reqwest::blocking::RequestBuilder> {
        let headers_clone = headers.clone();
        let mut req = request;
        for pair in headers_clone.pairs::<String, String>() {
            let (key, value) = pair?;
            req = req.header(&key, &value);
        }
        Ok(req)
    }

    /// Check if headers contain Content-Type
    fn has_content_type(headers: &Table) -> bool {
        let headers_clone = headers.clone();
        for (key, _) in headers_clone.pairs::<String, String>().flatten() {
            if key.to_lowercase() == "content-type" {
                return true;
            }
        }
        false
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Create rex.http module
pub fn create_http_module(lua: &Lua, client: HttpClient) -> LuaResult<Table<'_>> {
    let http_table = lua.create_table()?;

    // rex.http.get(url, headers?)
    http_table.set(
        "get",
        lua.create_function({
            let client = client.clone();
            move |lua, (url, headers): (String, Option<Table>)| {
                let (status, body) = client.get(&url, headers.as_ref())?;

                let result = lua.create_table()?;
                result.set("status", status)?;
                result.set("body", body.clone())?;
                result.set("ok", (200..300).contains(&status))?;

                // Try to parse as JSON
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                    let json_value = json_value_to_lua(lua, &parsed)?;
                    result.set("json", json_value)?;
                }

                Ok(result)
            }
        })?,
    )?;

    // rex.http.post(url, body?, headers?)
    http_table.set(
        "post",
        lua.create_function({
            let client = client.clone();
            move |lua, (url, body, headers): (String, Option<String>, Option<Table>)| {
                let (status, response_body) = client.post(&url, body, headers.as_ref())?;

                let result = lua.create_table()?;
                result.set("status", status)?;
                result.set("body", response_body.clone())?;
                result.set("ok", (200..300).contains(&status))?;

                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_body) {
                    let json_value = json_value_to_lua(lua, &parsed)?;
                    result.set("json", json_value)?;
                }

                Ok(result)
            }
        })?,
    )?;

    // rex.http.put(url, body?, headers?)
    http_table.set(
        "put",
        lua.create_function({
            let client = client.clone();
            move |lua, (url, body, headers): (String, Option<String>, Option<Table>)| {
                let (status, response_body) = client.put(&url, body, headers.as_ref())?;

                let result = lua.create_table()?;
                result.set("status", status)?;
                result.set("body", response_body.clone())?;
                result.set("ok", (200..300).contains(&status))?;

                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_body) {
                    let json_value = json_value_to_lua(lua, &parsed)?;
                    result.set("json", json_value)?;
                }

                Ok(result)
            }
        })?,
    )?;

    // rex.http.delete(url, headers?)
    http_table.set(
        "delete",
        lua.create_function({
            let client = client.clone();
            move |lua, (url, headers): (String, Option<Table>)| {
                let (status, response_body) = client.delete(&url, headers.as_ref())?;

                let result = lua.create_table()?;
                result.set("status", status)?;
                result.set("body", response_body.clone())?;
                result.set("ok", (200..300).contains(&status))?;

                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_body) {
                    let json_value = json_value_to_lua(lua, &parsed)?;
                    result.set("json", json_value)?;
                }

                Ok(result)
            }
        })?,
    )?;

    Ok(http_table)
}

/// Convert serde_json::Value to Lua Value (helper)
fn json_value_to_lua<'lua>(lua: &'lua Lua, value: &serde_json::Value) -> LuaResult<Value<'lua>> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(s) => lua.create_string(s).map(Value::String),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, val) in arr.iter().enumerate() {
                table.set(i + 1, json_value_to_lua(lua, val)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (key, val) in obj {
                table.set(key.clone(), json_value_to_lua(lua, val)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}
