//! HTTP Server - Background thread + polling pattern
//! Uses Lua Registry to store handler functions safely
//!
//! Architecture:
//! - Background thread: accepts HTTP connections, parses requests
//! - Lua main thread: polls requests, calls handlers (with pcall), submits responses
//! - Thread-safe via message queues

use mlua::{Function, Lua, RegistryKey, Result as LuaResult, Table, Value};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use tiny_http::{Header, Response, Server, StatusCode};
use tracing::{debug, error, info, warn};

/// HTTP Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub cors_origin: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_enabled: true,
            cors_origin: "*".to_string(),
        }
    }
}

/// Pending request from background thread
#[derive(Clone, Debug)]
pub struct PendingRequest {
    pub request_id: u64,
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Response back to background thread
#[derive(Clone, Debug)]
pub struct PendingResponse {
    pub request_id: u64,
    pub status: u16,
    pub body: String,
    pub content_type: String,
}

/// Shared state
struct ServerState {
    pending_requests: Vec<PendingRequest>,
    pending_responses: Vec<PendingResponse>,
    next_request_id: u64,
    running: bool,
}

impl ServerState {
    fn new() -> Self {
        Self {
            pending_requests: Vec::new(),
            pending_responses: Vec::new(),
            next_request_id: 1,
            running: false,
        }
    }
}

/// Route handler info
struct RouteInfo {
    handler_key: RegistryKey,
}

/// HTTP Server
pub struct HttpServer {
    config: ServerConfig,
    state: Arc<(Mutex<ServerState>, Condvar)>,
    routes: Arc<Mutex<HashMap<String, HashMap<String, RouteInfo>>>>,
    static_paths: Arc<Mutex<HashMap<String, String>>>,
}

impl HttpServer {
    /// Create new HTTP server
    pub fn new(config: ServerConfig) -> Self {
        info!("🌐 Creating HTTP Server at {}:{}", config.host, config.port);
        Self {
            config,
            state: Arc::new((Mutex::new(ServerState::new()), Condvar::new())),
            routes: Arc::new(Mutex::new(HashMap::new())),
            static_paths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add route with Lua handler (stores in registry)
    pub fn add_route(
        &self,
        lua: &Lua,
        method: &str,
        path: &str,
        handler: Function,
    ) -> LuaResult<()> {
        debug!("📝 Registering route: {} {}", method, path);

        let handler_key = lua.create_registry_value(handler)?;

        let mut routes = self.routes.lock().unwrap();
        let method_routes = routes.entry(method.to_uppercase()).or_default();
        method_routes.insert(path.to_string(), RouteInfo { handler_key });

        info!("📍 {} {}", method.to_uppercase(), path);
        Ok(())
    }

    /// Add static file path
    pub fn add_static_path(&self, url_path: &str, dir_path: &str) {
        debug!("📁 Registering static path: {} -> {}", url_path, dir_path);
        let mut static_paths = self.static_paths.lock().unwrap();
        static_paths.insert(url_path.to_string(), dir_path.to_string());
        info!("📁 Static: {} -> {}", url_path, dir_path);
    }

    /// Start server in background (non-blocking)
    pub fn start_background(&self) -> LuaResult<()> {
        {
            let (ref lock, _) = *self.state;
            let mut state = lock.lock().unwrap();
            if state.running {
                return Err(mlua::Error::RuntimeError(
                    "Server already running".to_string(),
                ));
            }
            state.running = true;
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let server = Server::http(&addr)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to start server: {}", e)))?;

        info!("🚀 HTTP Server running at http://{}", addr);

        let state = self.state.clone();
        let static_paths = self.static_paths.clone();
        let cors_enabled = self.config.cors_enabled;
        let cors_origin = self.config.cors_origin.clone();

        // Background thread
        thread::spawn(move || {
            info!("🧵 Background thread started");

            for request in server.incoming_requests() {
                debug!("📨 Received HTTP request");

                // Check if still running
                {
                    let (ref lock, _) = *state;
                    let state_guard = lock.lock().unwrap();
                    if !state_guard.running {
                        info!("🛑 Server stopped, breaking request loop");
                        break;
                    }
                }

                // Wrap request in Option to manage ownership
                let mut maybe_request = Some(request);

                let method;
                let url;
                let path;
                let query_string;
                let headers;
                let body;

                {
                    let req = maybe_request.as_mut().unwrap();
                    method = req.method().as_str().to_string();
                    url = req.url().to_string();

                    debug!("🔍 Parsing request: {} {}", method, url);

                    // Parse URL and query
                    let (p, q) = if let Some(idx) = url.find('?') {
                        (url[..idx].to_string(), url[idx + 1..].to_string())
                    } else {
                        (url.clone(), String::new())
                    };
                    path = p;
                    query_string = q;

                    // Parse headers
                    let mut h = HashMap::new();
                    for header in req.headers() {
                        h.insert(
                            header.field.to_string().to_lowercase(),
                            header.value.to_string(),
                        );
                    }
                    headers = h;

                    // Read body
                    let mut b = String::new();
                    if let Err(e) = req.as_reader().read_to_string(&mut b) {
                        error!("❌ Failed to read body: {}", e);
                    }
                    body = b;

                    debug!("📦 Request body: {} bytes", body.len());
                }

                // Check static files first
                let static_paths_guard = static_paths.lock().unwrap();
                for (url_prefix, dir_path) in static_paths_guard.iter() {
                    if path.starts_with(url_prefix) {
                        let file_path = path.replacen(url_prefix, dir_path, 1);
                        let file_path = if file_path.ends_with('/') {
                            format!("{}index.html", file_path)
                        } else {
                            file_path
                        };

                        debug!("🔍 Checking static file: {}", file_path);

                        if let Ok(content) = std::fs::read(&file_path) {
                            let content_type = Self::get_content_type(&file_path);
                            debug!("✅ Serving static file: {} ({})", file_path, content_type);

                            let mut response = Response::from_data(content).with_header(
                                Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                                    .unwrap(),
                            );

                            if cors_enabled {
                                response = response.with_header(
                                    Header::from_bytes(
                                        &b"Access-Control-Allow-Origin"[..],
                                        cors_origin.as_bytes(),
                                    )
                                    .unwrap(),
                                );
                            }

                            if let Some(req) = maybe_request.take() {
                                let _ = req.respond(response);
                            }
                            break;
                        } else {
                            debug!("❌ Static file not found: {}", file_path);
                        }
                    }
                }
                drop(static_paths_guard);

                // If request was consumed by static files, skip
                if maybe_request.is_none() {
                    continue;
                }

                // Push to pending requests
                let request_id = {
                    let (ref lock, ref cvar) = *state;
                    let mut state_guard = lock.lock().unwrap();
                    let id = state_guard.next_request_id;
                    state_guard.next_request_id += 1;

                    debug!("📤 Queuing request {} for Lua handler", id);

                    state_guard.pending_requests.push(PendingRequest {
                        request_id: id,
                        method: method.clone(),
                        path: path.clone(),
                        query: query_string,
                        headers,
                        body,
                    });

                    cvar.notify_one();
                    id
                };

                // Wait for response
                debug!("⏳ Waiting for response to request {}", request_id);
                let response = loop {
                    let (ref lock, ref cvar) = *state;
                    let mut state_guard = lock.lock().unwrap();

                    if let Some(pos) = state_guard
                        .pending_responses
                        .iter()
                        .position(|r| r.request_id == request_id)
                    {
                        let resp = state_guard.pending_responses.remove(pos);
                        debug!(
                            "✅ Got response for request {}: status {}",
                            request_id, resp.status
                        );
                        break resp;
                    }

                    if !state_guard.running {
                        warn!("⚠️ Server stopping, sending 503 for request {}", request_id);
                        break PendingResponse {
                            request_id,
                            status: 503,
                            body: "{\"error\":\"Server shutting down\"}".to_string(),
                            content_type: "application/json".to_string(),
                        };
                    }

                    let timeout = std::time::Duration::from_millis(100);
                    let _ = cvar.wait_timeout(state_guard, timeout);
                };

                // Send HTTP response
                let status = StatusCode(response.status);
                let mut http_response = Response::from_data(response.body.into_bytes())
                    .with_status_code(status)
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], response.content_type.as_bytes())
                            .unwrap(),
                    );

                if cors_enabled {
                    http_response = http_response
                        .with_header(
                            Header::from_bytes(
                                &b"Access-Control-Allow-Origin"[..],
                                cors_origin.as_bytes(),
                            )
                            .unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(
                                &b"Access-Control-Allow-Methods"[..],
                                &b"GET, POST, PUT, DELETE, OPTIONS"[..],
                            )
                            .unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(
                                &b"Access-Control-Allow-Headers"[..],
                                &b"Content-Type, Authorization"[..],
                            )
                            .unwrap(),
                        );
                }

                if let Some(req) = maybe_request.take() {
                    debug!("📤 Sending HTTP response for request {}", request_id);
                    let _ = req.respond(http_response);
                }
            }

            info!("🧵 Background thread exiting");
        });

        Ok(())
    }

    /// Poll for pending request
    pub fn poll_request(&self) -> Option<PendingRequest> {
        let (ref lock, _) = *self.state;
        let mut state = lock.lock().unwrap();

        if state.pending_requests.is_empty() {
            debug!("📭 No pending requests");
            return None;
        }

        let req = state.pending_requests.remove(0);
        debug!(
            "📬 Polled request {}: {} {}",
            req.request_id, req.method, req.path
        );
        Some(req)
    }

    /// Submit response
    pub fn submit_response(&self, response: PendingResponse) {
        debug!(
            "📝 Submitting response for request {}: status {}",
            response.request_id, response.status
        );
        let (ref lock, ref cvar) = *self.state;
        let mut state = lock.lock().unwrap();
        state.pending_responses.push(response);
        cvar.notify_one();
    }

    /// Get handler function from registry
    pub fn get_handler<'lua>(
        &self,
        lua: &'lua Lua,
        method: &str,
        path: &str,
    ) -> LuaResult<Option<Function<'lua>>> {
        debug!("🔍 Looking up handler for {} {}", method, path);

        let routes = self.routes.lock().unwrap();

        // Exact match
        if let Some(method_routes) = routes.get(method) {
            if let Some(route_info) = method_routes.get(path) {
                debug!("✅ Found exact match handler for {} {}", method, path);
                let handler = lua.registry_value::<Function>(&route_info.handler_key)?;
                return Ok(Some(handler));
            }

            // Pattern match
            for (route_path, route_info) in method_routes.iter() {
                if Self::match_route(route_path, path) {
                    debug!("✅ Found pattern match handler: {} -> {}", path, route_path);
                    let handler = lua.registry_value::<Function>(&route_info.handler_key)?;
                    return Ok(Some(handler));
                }
            }
        }

        debug!("❌ No handler found for {} {}", method, path);
        Ok(None)
    }

    /// Simple route pattern matching
    fn match_route(pattern: &str, path: &str) -> bool {
        if pattern == path {
            return true;
        }
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        if pattern_parts.len() != path_parts.len() {
            return false;
        }
        for (p, r) in pattern_parts.iter().zip(path_parts.iter()) {
            if p.starts_with(':') || p.starts_with('*') {
                continue;
            }
            if p != r {
                return false;
            }
        }
        true
    }

    /// Get content type
    fn get_content_type(path: &str) -> &'static str {
        match path.rsplit('.').next() {
            Some("html") | Some("htm") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("txt") => "text/plain",
            Some("xml") => "application/xml",
            Some("pdf") => "application/pdf",
            _ => "application/octet-stream",
        }
    }

    /// Stop server
    pub fn stop(&self) {
        info!("🛑 Stopping HTTP Server");
        let (ref lock, ref cvar) = *self.state;
        let mut state = lock.lock().unwrap();
        state.running = false;
        cvar.notify_all();
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        let (ref lock, _) = *self.state;
        let state = lock.lock().unwrap();
        state.running
    }

    /// Get address
    pub fn address(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }

    /// Get routes count
    pub fn routes_count(&self) -> usize {
        let routes = self.routes.lock().unwrap();
        routes.values().map(|m| m.len()).sum()
    }
}

/// Create rex.http.server module
pub fn create_http_server_module(lua: &Lua) -> LuaResult<Table<'_>> {
    let server_table = lua.create_table()?;

    // rex.http.server.create(config?)
    server_table.set(
        "create",
        lua.create_function(|lua, config: Option<Table>| {
            let server_config = if let Some(cfg) = config {
                ServerConfig {
                    host: cfg
                        .get::<_, Option<String>>("host")?
                        .unwrap_or_else(|| "127.0.0.1".to_string()),
                    port: cfg.get::<_, Option<u16>>("port")?.unwrap_or(8080),
                    cors_enabled: cfg.get::<_, Option<bool>>("cors")?.unwrap_or(true),
                    cors_origin: cfg
                        .get::<_, Option<String>>("cors_origin")?
                        .unwrap_or_else(|| "*".to_string()),
                }
            } else {
                ServerConfig::default()
            };

            let server = HttpServer::new(server_config.clone());

            let result = lua.create_table()?;
            let server_ptr = Arc::into_raw(Arc::new(server)) as usize;
            result.set("__ptr", server_ptr)?;
            result.set(
                "address",
                format!("{}:{}", server_config.host, server_config.port),
            )?;

            Ok(Value::Table(result))
        })?,
    )?;

    // Helper to get server
    fn get_server(conn: &Table) -> LuaResult<Arc<HttpServer>> {
        let ptr: usize = conn.get("__ptr")?;
        if ptr == 0 {
            return Err(mlua::Error::RuntimeError("Invalid server".to_string()));
        }
        let server = unsafe { Arc::from_raw(ptr as *const HttpServer) };
        let server_clone = server.clone();
        std::mem::forget(server);
        Ok(server_clone)
    }

    // rex.http.server.get(server, path, handler)
    server_table.set(
        "get",
        lua.create_function(
            |lua, (server_table, path, handler): (Table, String, Function)| {
                let server = get_server(&server_table)?;
                server.add_route(lua, "GET", &path, handler)
            },
        )?,
    )?;

    // rex.http.server.post(server, path, handler)
    server_table.set(
        "post",
        lua.create_function(
            |lua, (server_table, path, handler): (Table, String, Function)| {
                let server = get_server(&server_table)?;
                server.add_route(lua, "POST", &path, handler)
            },
        )?,
    )?;

    // rex.http.server.put(server, path, handler)
    server_table.set(
        "put",
        lua.create_function(
            |lua, (server_table, path, handler): (Table, String, Function)| {
                let server = get_server(&server_table)?;
                server.add_route(lua, "PUT", &path, handler)
            },
        )?,
    )?;

    // rex.http.server.delete(server, path, handler)
    server_table.set(
        "delete",
        lua.create_function(
            |lua, (server_table, path, handler): (Table, String, Function)| {
                let server = get_server(&server_table)?;
                server.add_route(lua, "DELETE", &path, handler)
            },
        )?,
    )?;

    // rex.http.server.static(server, url_path, dir_path)
    server_table.set(
        "static",
        lua.create_function(
            |_, (server_table, url_path, dir_path): (Table, String, String)| {
                let server = get_server(&server_table)?;
                server.add_static_path(&url_path, &dir_path);
                Ok(())
            },
        )?,
    )?;

    // rex.http.server.start(server)
    server_table.set(
        "start",
        lua.create_function(|_, server_table: Table| {
            let server = get_server(&server_table)?;
            server.start_background()
        })?,
    )?;

    // rex.http.server.stop(server)
    server_table.set(
        "stop",
        lua.create_function(|_, server_table: Table| {
            let server = get_server(&server_table)?;
            server.stop();
            Ok(())
        })?,
    )?;

    // rex.http.server.is_running(server)
    server_table.set(
        "is_running",
        lua.create_function(|_, server_table: Table| {
            let server = get_server(&server_table)?;
            Ok(server.is_running())
        })?,
    )?;

    // rex.http.server.routes_count(server)
    server_table.set(
        "routes_count",
        lua.create_function(|_, server_table: Table| {
            let server = get_server(&server_table)?;
            Ok(server.routes_count())
        })?,
    )?;

    // rex.http.server.poll(server) - Poll for pending request
    server_table.set(
        "poll",
        lua.create_function(|lua, server_table: Table| {
            let server = get_server(&server_table)?;
            if let Some(req) = server.poll_request() {
                let result = lua.create_table()?;
                result.set("request_id", req.request_id)?;
                result.set("method", req.method)?;
                result.set("path", req.path)?;
                result.set("query", req.query)?;
                result.set("body", req.body)?;

                let headers_table = lua.create_table()?;
                for (key, value) in req.headers {
                    headers_table.set(key, value)?;
                }
                result.set("headers", headers_table)?;

                Ok(Value::Table(result))
            } else {
                Ok(Value::Nil)
            }
        })?,
    )?;

    // rex.http.server.respond(server, request_id, body, status?, content_type?)
    server_table.set(
        "respond",
        lua.create_function(
            |_,
             (server_table, request_id, body, status, content_type): (
                Table,
                u64,
                String,
                Option<u16>,
                Option<String>,
            )| {
                let server = get_server(&server_table)?;
                server.submit_response(PendingResponse {
                    request_id,
                    status: status.unwrap_or(200),
                    body,
                    content_type: content_type.unwrap_or_else(|| "application/json".to_string()),
                });
                Ok(())
            },
        )?,
    )?;

    // ✅ REMOVED: rex.http.server.handle() - causes panic
    // Use poll() + respond() in Lua with pcall instead

    Ok(server_table)
}
