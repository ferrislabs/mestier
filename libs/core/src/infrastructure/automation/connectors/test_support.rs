//! A local HTTP server for connector tests, bound to a dynamically allocated
//! port (never a fixed one — see #210 on test contention) and speaking just
//! enough HTTP/1.1 to drive the network connectors: request line, headers,
//! a `content-length` body, and persistent connections (the Odoo connectors
//! make two calls per action, and a real client may reuse one socket for
//! both).
//!
//! Every server here lives on `127.0.0.1`, which the guarded resolver
//! refuses unconditionally regardless of policy (loopback is never a
//! legitimate webhook or API target). Production connectors reach it only
//! through a `#[cfg(test)]` seam that bypasses the guard — the guard itself
//! is proved once, in `infrastructure::automation::webhook::resolver`, not
//! re-proved against a server that cannot represent what it guards.
#![cfg(test)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// One HTTP request captured by [`TestServer`], parsed just enough for
/// assertions.
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub method: String,
    pub path: String,
    /// Header names lower-cased, the same normalization a receiver would
    /// apply — case must never be what an assertion depends on.
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl CapturedRequest {
    pub fn body_json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("captured body is valid JSON")
    }
}

/// What the server answers a captured request with.
#[derive(Clone)]
pub struct StubResponse {
    status: u16,
    reason: &'static str,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl StubResponse {
    pub fn status(status: u16) -> Self {
        Self {
            status,
            reason: reason_phrase(status),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn json(status: u16, body: &Value) -> Self {
        Self::status(status)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_vec(body).expect("test body serializes"))
    }

    pub fn text(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self::status(status)
            .with_header("content-type", "text/plain")
            .with_body(body.into())
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    fn render(&self) -> Vec<u8> {
        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status, self.reason);
        let mut has_content_length = false;
        for (name, value) in &self.headers {
            if name.eq_ignore_ascii_case("content-length") {
                has_content_length = true;
            }
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        if !has_content_length {
            head.push_str(&format!("content-length: {}\r\n", self.body.len()));
        }
        head.push_str("\r\n");

        let mut rendered = head.into_bytes();
        rendered.extend_from_slice(&self.body);
        rendered
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        302 => "Found",
        308 => "Permanent Redirect",
        401 => "Unauthorized",
        404 => "Not Found",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Response",
    }
}

/// A local server driven by a handler that decides the response for every
/// request it captures, in order.
pub struct TestServer {
    pub url: String,
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
    _task: JoinHandle<()>,
}

impl TestServer {
    /// Every request gets the same response.
    pub async fn respond_always(response: StubResponse) -> Self {
        Self::start(move |_| response.clone()).await
    }

    /// `handler` is called once per captured request, in the order the
    /// requests arrive — the shape a two-step protocol like Odoo's
    /// authenticate-then-execute needs.
    pub async fn start<F>(handler: F) -> Self
    where
        F: Fn(&CapturedRequest) -> StubResponse + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binding a dynamic port never fails locally");
        let address = listener
            .local_addr()
            .expect("a bound listener has an address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let handler = Arc::new(handler);

        let task = tokio::spawn(async move {
            loop {
                let Ok((socket, _)) = listener.accept().await else {
                    return;
                };
                let handler = Arc::clone(&handler);
                let captured = Arc::clone(&captured);
                tokio::spawn(serve_connection(socket, handler, captured));
            }
        });

        Self {
            url: format!("http://{address}"),
            requests,
            _task: task,
        }
    }

    /// Requests captured so far, in arrival order.
    pub fn requests(&self) -> Vec<CapturedRequest> {
        self.requests
            .lock()
            .expect("test server mutex poisoned")
            .clone()
    }

    pub fn last_request(&self) -> CapturedRequest {
        self.requests()
            .into_iter()
            .next_back()
            .expect("at least one request was captured")
    }
}

async fn serve_connection<F>(
    mut socket: tokio::net::TcpStream,
    handler: Arc<F>,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
) where
    F: Fn(&CapturedRequest) -> StubResponse + Send + Sync + 'static,
{
    let mut buffer: Vec<u8> = Vec::new();
    let mut read_chunk = [0u8; 8192];

    loop {
        let Some(request) = read_one_request(&mut socket, &mut buffer, &mut read_chunk).await
        else {
            return;
        };

        // `spawn_blocking`: a handler simulating a slow endpoint (the
        // timeout test) calls `std::thread::sleep`, which would otherwise
        // freeze the whole reactor under a single-threaded test runtime and
        // starve the very timer the client's timeout depends on.
        let for_handler = request.clone();
        let handler_for_call = Arc::clone(&handler);
        let response = tokio::task::spawn_blocking(move || handler_for_call(&for_handler))
            .await
            .expect("the response handler must not panic");
        captured
            .lock()
            .expect("test server mutex poisoned")
            .push(request);

        if socket.write_all(&response.render()).await.is_err() {
            return;
        }
        if socket.flush().await.is_err() {
            return;
        }
    }
}

/// Reads one HTTP/1.1 request off `socket`, using `buffer` as look-ahead
/// storage across calls so a second request pipelined behind the first is
/// never dropped. Returns `None` once the peer closes the connection.
async fn read_one_request(
    socket: &mut tokio::net::TcpStream,
    buffer: &mut Vec<u8>,
    read_chunk: &mut [u8; 8192],
) -> Option<CapturedRequest> {
    let header_end = loop {
        if let Some(position) = find_header_end(buffer) {
            break position;
        }
        let read = socket.read(read_chunk).await.ok()?;
        if read == 0 {
            return None;
        }
        buffer.extend_from_slice(&read_chunk[..read]);
    };

    let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split(' ');
    let method = parts.next().unwrap_or_default().to_owned();
    let path = parts.next().unwrap_or_default().to_owned();

    let headers: HashMap<String, String> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_lowercase(), value.trim().to_owned()))
        .collect();

    let content_length: usize = headers
        .get("content-length")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buffer.len() < body_start + content_length {
        let read = socket.read(read_chunk).await.ok()?;
        if read == 0 {
            return None;
        }
        buffer.extend_from_slice(&read_chunk[..read]);
    }

    let body = buffer[body_start..body_start + content_length].to_vec();
    buffer.drain(..body_start + content_length);

    Some(CapturedRequest {
        method,
        path,
        headers,
        body,
    })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_server_captures_the_method_path_headers_and_body() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let client = reqwest::Client::new();

        client
            .post(format!("{}/hook", server.url))
            .header("x-probe", "42")
            .body(r#"{"a":1}"#)
            .send()
            .await
            .unwrap();

        let request = server.last_request();
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/hook");
        assert_eq!(
            request.headers.get("x-probe").map(String::as_str),
            Some("42")
        );
        assert_eq!(request.body_json(), serde_json::json!({ "a": 1 }));
    }

    #[tokio::test]
    async fn a_server_answers_the_configured_status_and_body() {
        let server =
            TestServer::respond_always(StubResponse::json(422, &serde_json::json!({"e": "x"})))
                .await;
        let client = reqwest::Client::new();

        let response = client.get(&server.url).send().await.unwrap();

        assert_eq!(response.status().as_u16(), 422);
        assert_eq!(
            response.json::<Value>().await.unwrap(),
            serde_json::json!({"e": "x"})
        );
    }

    /// The Odoo connectors make two calls per action; a real client is free
    /// to reuse one socket for both, and the server must not drop the
    /// second request when it does.
    #[tokio::test]
    async fn a_server_handles_two_requests_on_the_same_connection() {
        let calls = Arc::new(Mutex::new(0u32));
        let server = TestServer::start(move |_| {
            let mut count = calls.lock().unwrap();
            *count += 1;
            StubResponse::json(200, &serde_json::json!({ "call": *count }))
        })
        .await;
        let client = reqwest::Client::new();

        let first = client
            .post(&server.url)
            .body("{}")
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        let second = client
            .post(&server.url)
            .body("{}")
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();

        assert_eq!(first, serde_json::json!({ "call": 1 }));
        assert_eq!(second, serde_json::json!({ "call": 2 }));
        assert_eq!(server.requests().len(), 2);
    }
}
