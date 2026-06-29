//! WebSocket Client Implementation - Thread-safe version
//!
//! Uses message queue pattern to avoid thread safety issues with Lua callbacks

use futures_util::{SinkExt, StreamExt};
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

/// WebSocket connection state
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl ConnectionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionState::Disconnected => "disconnected",
            ConnectionState::Connecting => "connecting",
            ConnectionState::Connected => "connected",
            ConnectionState::Error(_) => "error",
        }
    }
}

/// WebSocket client wrapper - Thread-safe
#[derive(Clone)]
pub struct WebSocketClient {
    url: String,
    state: Arc<Mutex<ConnectionState>>,
    incoming_messages: Arc<Mutex<VecDeque<String>>>,
    outgoing_messages: Arc<Mutex<VecDeque<String>>>,
    errors: Arc<Mutex<Vec<String>>>,
    runtime: Arc<Runtime>,
}

impl WebSocketClient {
    /// Create new WebSocket client
    pub fn new(url: &str) -> LuaResult<Self> {
        debug!("🔌 Creating WebSocket client for: {}", url);

        let runtime = Runtime::new()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create runtime: {}", e)))?;

        Ok(Self {
            url: url.to_string(),
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            incoming_messages: Arc::new(Mutex::new(VecDeque::new())),
            outgoing_messages: Arc::new(Mutex::new(VecDeque::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
            runtime: Arc::new(runtime),
        })
    }

    /// Connect to WebSocket server (non-blocking)
    pub fn connect(&self) -> LuaResult<()> {
        info!("🔌 Connecting to WebSocket: {}", self.url);

        {
            let mut state = self.state.lock().unwrap();
            *state = ConnectionState::Connecting;
        }

        let url = self.url.clone();
        let state = self.state.clone();
        let incoming = self.incoming_messages.clone();
        let outgoing = self.outgoing_messages.clone();
        let errors = self.errors.clone();

        let runtime = self.runtime.clone();

        // Spawn connection in background thread
        thread::spawn(move || {
            runtime.block_on(async move {
                match connect_async(&url).await {
                    Ok((ws_stream, _response)) => {
                        info!("✅ WebSocket connected to: {}", url);

                        {
                            let mut state_lock = state.lock().unwrap();
                            *state_lock = ConnectionState::Connected;
                        }

                        let (mut write, mut read) = ws_stream.split();

                        // Spawn message sender task
                        let out_msgs = outgoing.clone();
                        let write_state = state.clone();
                        let write_errors = errors.clone();

                        tokio::spawn(async move {
                            loop {
                                let msg = {
                                    let mut queue = out_msgs.lock().unwrap();
                                    queue.pop_front()
                                };

                                if let Some(message) = msg {
                                    debug!("📤 Sending: {}", message);
                                    if let Err(e) = write.send(Message::Text(message)).await {
                                        error!("Failed to send message: {}", e);
                                        let mut err_lock = write_errors.lock().unwrap();
                                        err_lock.push(format!("Send error: {}", e));

                                        let mut state_lock = write_state.lock().unwrap();
                                        *state_lock = ConnectionState::Error(e.to_string());
                                        break;
                                    }
                                } else {
                                    tokio::time::sleep(Duration::from_millis(10)).await;
                                }

                                let is_connected = {
                                    let state_lock = write_state.lock().unwrap();
                                    *state_lock == ConnectionState::Connected
                                };

                                if !is_connected {
                                    break;
                                }
                            }
                        });

                        // Message receiver loop
                        while let Some(msg) = read.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    debug!("📥 Received: {}", text);
                                    let mut queue = incoming.lock().unwrap();
                                    queue.push_back(text);
                                }
                                Ok(Message::Binary(data)) => {
                                    debug!("📥 Received binary: {} bytes", data.len());
                                    let mut queue = incoming.lock().unwrap();
                                    queue.push_back(String::from_utf8_lossy(&data).to_string());
                                }
                                Ok(Message::Ping(_)) => {
                                    debug!("📥 Received ping");
                                }
                                Ok(Message::Pong(_)) => {
                                    debug!("📥 Received pong");
                                }
                                Ok(Message::Close(_)) => {
                                    info!("🔌 WebSocket closed by server");
                                    let mut state_lock = state.lock().unwrap();
                                    *state_lock = ConnectionState::Disconnected;
                                    break;
                                }
                                Ok(Message::Frame(_)) => {}
                                Err(e) => {
                                    error!("WebSocket error: {}", e);
                                    let mut err_lock = errors.lock().unwrap();
                                    err_lock.push(format!("Connection error: {}", e));

                                    let mut state_lock = state.lock().unwrap();
                                    *state_lock = ConnectionState::Error(e.to_string());
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to connect: {}", e);
                        let mut state_lock = state.lock().unwrap();
                        *state_lock = ConnectionState::Error(e.to_string());

                        let mut err_lock = errors.lock().unwrap();
                        err_lock.push(format!("Connection failed: {}", e));
                    }
                }
            });
        });

        thread::sleep(Duration::from_millis(100));

        Ok(())
    }

    /// Send message (non-blocking, adds to queue)
    pub fn send(&self, message: &str) -> LuaResult<()> {
        let state = self.state.lock().unwrap();
        if *state != ConnectionState::Connected {
            return Err(mlua::Error::RuntimeError(format!(
                "WebSocket not connected (state: {})",
                state.as_str()
            )));
        }
        drop(state);

        debug!("📤 Queuing message: {}", message);
        let mut queue = self.outgoing_messages.lock().unwrap();
        queue.push_back(message.to_string());

        Ok(())
    }

    /// Poll for incoming messages (returns next message or nil)
    pub fn poll_message(&self) -> Option<String> {
        let mut queue = self.incoming_messages.lock().unwrap();
        queue.pop_front()
    }

    /// Get all pending messages
    pub fn poll_all_messages(&self) -> Vec<String> {
        let mut queue = self.incoming_messages.lock().unwrap();
        queue.drain(..).collect()
    }

    /// Get connection state
    pub fn get_state(&self) -> String {
        let state = self.state.lock().unwrap();
        state.as_str().to_string()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        let state = self.state.lock().unwrap();
        *state == ConnectionState::Connected
    }

    /// Get pending errors
    pub fn poll_errors(&self) -> Vec<String> {
        let mut errors = self.errors.lock().unwrap();
        errors.drain(..).collect()
    }

    /// Close connection
    pub fn close(&self) -> LuaResult<()> {
        info!("🔌 Closing WebSocket connection");
        let mut state = self.state.lock().unwrap();
        *state = ConnectionState::Disconnected;
        Ok(())
    }

    /// Get URL
    pub fn get_url(&self) -> String {
        self.url.clone()
    }
}

/// Helper function to get client from connection table
/// ✅ Đây là hàm ĐỘC LẬP, không phải closure - không bị lỗi lifetime!
fn get_client_from_table(conn: &Table) -> LuaResult<Arc<WebSocketClient>> {
    let ptr: usize = conn.get("__ptr")?;
    if ptr == 0 {
        return Err(mlua::Error::RuntimeError("Invalid connection".to_string()));
    }
    // Safety: We created this Arc in connect() and it's still valid
    let client = unsafe { Arc::from_raw(ptr as *const WebSocketClient) };
    let client_clone = client.clone();
    std::mem::forget(client);
    Ok(client_clone)
}

/// Create rex.websocket module
pub fn create_websocket_module(lua: &Lua) -> LuaResult<Table<'_>> {
    let ws_table = lua.create_table()?;

    // rex.websocket.connect(url)
    ws_table.set(
        "connect",
        lua.create_function(|lua, url: String| {
            let client = WebSocketClient::new(&url)?;
            client.connect()?;

            let conn = lua.create_table()?;

            // Store client as raw pointer
            let client_ptr = Arc::into_raw(Arc::new(client)) as usize;
            conn.set("__ptr", client_ptr)?;

            conn.set("url", url)?;
            conn.set("connected", false)?;
            conn.set("state", "connecting")?;

            Ok(Value::Table(conn))
        })?,
    )?;

    // rex.websocket.send(connection, message)
    ws_table.set(
        "send",
        lua.create_function(|_, (conn, message): (Table, String)| {
            let client = get_client_from_table(&conn)?;
            client.send(&message)
        })?,
    )?;

    // rex.websocket.poll(connection) - Get next message
    ws_table.set(
        "poll",
        lua.create_function(|_, conn: Table| {
            let client = get_client_from_table(&conn)?;
            Ok(client.poll_message())
        })?,
    )?;

    // rex.websocket.poll_all(connection) - Get all pending messages
    ws_table.set(
        "poll_all",
        lua.create_function(|lua, conn: Table| {
            let client = get_client_from_table(&conn)?;
            let messages = client.poll_all_messages();

            let table = lua.create_table()?;
            for (i, msg) in messages.into_iter().enumerate() {
                table.set(i + 1, msg)?;
            }
            Ok(Value::Table(table))
        })?,
    )?;

    // rex.websocket.is_connected(connection)
    ws_table.set(
        "is_connected",
        lua.create_function(|_, conn: Table| {
            let client = get_client_from_table(&conn)?;
            Ok(client.is_connected())
        })?,
    )?;

    // rex.websocket.get_state(connection)
    ws_table.set(
        "get_state",
        lua.create_function(|_, conn: Table| {
            let client = get_client_from_table(&conn)?;
            Ok(client.get_state())
        })?,
    )?;

    // rex.websocket.poll_errors(connection)
    ws_table.set(
        "poll_errors",
        lua.create_function(|lua, conn: Table| {
            let client = get_client_from_table(&conn)?;
            let errors = client.poll_errors();

            let table = lua.create_table()?;
            for (i, err) in errors.into_iter().enumerate() {
                table.set(i + 1, err)?;
            }
            Ok(Value::Table(table))
        })?,
    )?;

    // rex.websocket.close(connection)
    ws_table.set(
        "close",
        lua.create_function(|_, conn: Table| {
            let client = get_client_from_table(&conn)?;
            client.close()
        })?,
    )?;

    Ok(ws_table)
}
