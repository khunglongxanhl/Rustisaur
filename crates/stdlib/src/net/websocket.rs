//! WebSocket client support.

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::error::{Result, StdlibError};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket client connection.
pub struct WebSocket {
    write: futures_util::stream::SplitSink<WsStream, Message>,
    read: futures_util::stream::SplitStream<WsStream>,
}

impl WebSocket {
    /// Connect to a WebSocket URL.
    pub async fn connect(url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| StdlibError::WebSocket(e.to_string()))?;
        let (write, read) = ws_stream.split();
        Ok(Self { write, read })
    }

    /// Send a text message.
    pub async fn send(&mut self, message: &str) -> Result<()> {
        self.write
            .send(Message::Text(message.to_string()))
            .await
            .map_err(|e| StdlibError::WebSocket(e.to_string()))
    }

    /// Receive the next text message.
    pub async fn receive(&mut self) -> Result<String> {
        if let Some(msg) = self.read.next().await {
            match msg.map_err(|e| StdlibError::WebSocket(e.to_string()))? {
                Message::Text(text) => Ok(text),
                Message::Close(_) => Err(StdlibError::WebSocket("Connection closed".into())),
                _ => Err(StdlibError::WebSocket("Invalid message type".into())),
            }
        } else {
            Err(StdlibError::WebSocket("Connection closed".into()))
        }
    }
}
