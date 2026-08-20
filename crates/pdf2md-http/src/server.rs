use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use utoipa::OpenApi;
use pdf2md_core::Converter;
use crate::models::{AsyncTaskResponse, ConversionResponse, ErrorResponse, HealthResponse};
use crate::openapi::ApiDoc;
use crate::task_manager::TaskManager;

pub struct HttpServer {
    pub host: String,
    pub port: u16,
    pub converter: Arc<Converter>,
    pub task_manager: Arc<TaskManager>,
}

impl HttpServer {
    pub fn new(host: String, port: u16, converter: Arc<Converter>) -> Self {
        Self {
            host,
            port,
            converter,
            task_manager: Arc::new(TaskManager::new()),
        }
    }

    pub fn start(&self) -> std::io::Result<()> {
        let bind_addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&bind_addr)?;
        println!("pdf2md HTTP microservice listening on http://{}", bind_addr);
        println!("OpenAPI 3.0 docs available at http://{}/api-docs/openapi.json", bind_addr);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let conv = Arc::clone(&self.converter);
                    let tm = Arc::clone(&self.task_manager);
                    std::thread::spawn(move || {
                        let _ = handle_client(stream, conv, tm);
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }

        Ok(())
    }
}

pub fn handle_client(
    mut stream: TcpStream,
    converter: Arc<Converter>,
    task_manager: Arc<TaskManager>,
) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];
    let mut content_length = 0usize;
    let mut header_end_idx = None;

    loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);

        if header_end_idx.is_none() {
            if let Some(pos) = find_subsequence(&buffer, b"\r\n\r\n") {
                header_end_idx = Some(pos + 4);
                let header_str = String::from_utf8_lossy(&buffer[..pos]);
                for line in header_str.lines() {
                    if line.to_lowercase().starts_with("content-length:") {
                        if let Some(val) = line.split(':').nth(1) {
                            content_length = val.trim().parse::<usize>().unwrap_or(0);
                        }
                    }
                }
            }
        }

        if let Some(h_end) = header_end_idx {
            if buffer.len() >= h_end + content_length {
                break;
            }
        }
    }

    if let Some(h_end) = header_end_idx {
        let header_str = String::from_utf8_lossy(&buffer[..h_end]);
        let first_line = header_str.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return send_response(&mut stream, 400, "Bad Request", "text/plain", "Bad Request");
        }

        let method = parts[0];
        let uri = parts[1];

        // 1. GET /health
        if method == "GET" && uri.starts_with("/health") {
            let res = HealthResponse {
                status: "ok".to_string(),
                version: "0.1.0".to_string(),
            };
            let body = serde_json::to_string(&res).unwrap();
            return send_response(&mut stream, 200, "OK", "application/json", &body);
        }

        // 2. GET /api-docs/openapi.json
        if method == "GET" && (uri.starts_with("/api-docs/openapi.json") || uri.starts_with("/openapi.json")) {
            let openapi = ApiDoc::openapi();
            let body = openapi.to_pretty_json().unwrap_or_else(|_| "{}".to_string());
            return send_response(&mut stream, 200, "OK", "application/json", &body);
        }

        // 3. GET /status/{task_id}
        if method == "GET" && uri.starts_with("/status/") {
            let task_id = uri.trim_start_matches("/status/").split('?').next().unwrap_or("");
            if let Some(status_res) = task_manager.get_task_status(task_id) {
                let body = serde_json::to_string(&status_res).unwrap();
                return send_response(&mut stream, 200, "OK", "application/json", &body);
            } else {
                let err = ErrorResponse {
                    error: format!("Task '{}' not found", task_id),
                };
                let body = serde_json::to_string(&err).unwrap();
                return send_response(&mut stream, 404, "Not Found", "application/json", &body);
            }
        }

        // 4. POST /convert
        if method == "POST" && uri.starts_with("/convert") {
            let body_bytes = &buffer[h_end..h_end + content_length];
            let is_async = uri.contains("async=true")
                || header_str.to_lowercase().contains("x-async: true")
                || body_bytes.len() > 2 * 1024 * 1024;

            if is_async {
                let task_id = task_manager.submit_task(body_bytes.to_vec(), Arc::clone(&converter));
                let async_res = AsyncTaskResponse {
                    task_id: task_id.clone(),
                    status: "processing".to_string(),
                    status_url: format!("/status/{}", task_id),
                };
                let body = serde_json::to_string(&async_res).unwrap();
                return send_response(&mut stream, 202, "Accepted", "application/json", &body);
            } else {
                match converter.convert_bytes(body_bytes) {
                    Ok(result) => {
                        let conv_res = ConversionResponse {
                            markdown: result.markdown,
                            confidence: result.diagnostics.overall_confidence,
                            total_pages: result.diagnostics.total_pages,
                            tables_detected: result.diagnostics.tables_detected,
                            images_extracted: result.diagnostics.images_extracted,
                        };
                        let body = serde_json::to_string(&conv_res).unwrap();
                        return send_response(&mut stream, 200, "OK", "application/json", &body);
                    }
                    Err(e) => {
                        let err = ErrorResponse {
                            error: e.to_string(),
                        };
                        let body = serde_json::to_string(&err).unwrap();
                        return send_response(&mut stream, 400, "Bad Request", "application/json", &body);
                    }
                }
            }
        }
    }

    let not_found = ErrorResponse {
        error: "Route not found".to_string(),
    };
    let body = serde_json::to_string(&not_found).unwrap();
    send_response(&mut stream, 404, "Not Found", "application/json", &body)
}

fn send_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_code,
        status_text,
        content_type,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}
