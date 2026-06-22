//! HTTP/HTTPS client.

use reqwest::{Client, Response};

use crate::error::{Result, StdlibError};

/// HTTP response wrapper.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub content_type: Option<String>,
}

/// Async HTTP client.
pub struct HttpClient {
    client: Client,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    /// Create a new HTTP client with default settings.
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Perform a GET request.
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| StdlibError::Http(e.to_string()))?;
        self.process_response(response).await
    }

    /// Perform a POST request with JSON body.
    pub async fn post(&self, url: &str, body: &str) -> Result<HttpResponse> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| StdlibError::Http(e.to_string()))?;
        self.process_response(response).await
    }

    async fn process_response(&self, response: Response) -> Result<HttpResponse> {
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let body = response
            .text()
            .await
            .map_err(|e| StdlibError::Http(e.to_string()))?;

        Ok(HttpResponse {
            status,
            body,
            content_type,
        })
    }
}
