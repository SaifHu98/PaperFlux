use pdf2md_core::{Config, Converter};
use pdf2md_http::{ApiDoc, HttpServer, TaskManager, TaskStatus};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use utoipa::OpenApi;

fn generate_test_pdf() -> Vec<u8> {
    let text = "BT\n/F1 12 Tf\n72 700 Td\n(HTTP Async Integration Test) Tj\n0 -20 Td\n(Arabic: \\xd8\\xaa\\xd8\\xac\\xd8\\xb1\\xd8\\xa8\\xd8\\xa9) Tj\nET\n";
    let len = text.len();
    format!(
        "%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>\nendobj\n4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000300 00000 n \ntrailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n450\n%%EOF\n",
        len, text
    ).into_bytes()
}

#[test]
fn test_openapi_schema_generation_and_components() {
    let openapi = ApiDoc::openapi();
    let json_str = openapi
        .to_pretty_json()
        .expect("OpenAPI should serialize to JSON");

    assert!(json_str.contains("\"openapi\": \"3."));
    assert!(json_str.contains("/health"));
    assert!(json_str.contains("/convert"));
    assert!(json_str.contains("/status/{task_id}"));
    assert!(json_str.contains("ConversionResponse"));
    assert!(json_str.contains("AsyncTaskResponse"));
    assert!(json_str.contains("TaskStatusResponse"));
}

#[test]
fn test_task_manager_async_lifecycle() {
    let tm = TaskManager::new();
    let config = Config::builder().build();
    let converter = Arc::new(Converter::new(config));
    let pdf_bytes = generate_test_pdf();

    let task_id = tm.submit_task(pdf_bytes, converter);
    assert!(!task_id.is_empty());
    assert!(task_id.starts_with("task_"));

    // Poll until completed (timeout after 5 seconds)
    let mut completed = false;
    for _ in 0..50 {
        if let Some(status_res) = tm.get_task_status(&task_id) {
            if status_res.status == TaskStatus::Completed {
                assert!(status_res.result.is_some());
                let res = status_res.result.unwrap();
                assert_eq!(res.total_pages, 1);
                assert!(res.markdown.contains("HTTP Async Integration"));
                completed = true;
                break;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    assert!(completed, "Task should complete within timeout");
}

fn send_http_request(port: u16, request: &str, body: &[u8]) -> (u16, String) {
    let mut stream =
        TcpStream::connect(format!("127.0.0.1:{}", port)).expect("Should connect to server");
    stream.write_all(request.as_bytes()).unwrap();
    if !body.is_empty() {
        stream.write_all(body).unwrap();
    }

    let mut response_bytes = Vec::new();
    let mut buf = [0u8; 4096];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        response_bytes.extend_from_slice(&buf[..n]);
    }

    let resp_str = String::from_utf8_lossy(&response_bytes).to_string();
    let status_line = resp_str.lines().next().unwrap_or("");
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);

    let body_part = resp_str.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status_code, body_part)
}

#[test]
fn test_http_server_endpoints_and_async_workflow() {
    // Find free port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let config = Config::builder().build();
    let converter = Arc::new(Converter::new(config));
    let server = HttpServer::new("127.0.0.1".to_string(), port, converter);

    thread::spawn(move || {
        let _ = server.start();
    });
    thread::sleep(Duration::from_millis(100));

    // 1. GET /health
    let (status, body) = send_http_request(
        port,
        &format!(
            "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            port
        ),
        &[],
    );
    assert_eq!(status, 200);
    assert!(body.contains("\"status\":\"ok\""));

    // 2. GET /api-docs/openapi.json
    let (status, body) = send_http_request(
        port,
        &format!("GET /api-docs/openapi.json HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port),
        &[],
    );
    assert_eq!(status, 200);
    assert!(body.contains("\"openapi\": \"3."));

    // 3. POST /convert (Sync)
    let pdf_bytes = generate_test_pdf();
    let req_sync = format!(
        "POST /convert HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/pdf\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        port, pdf_bytes.len()
    );
    let (status, body) = send_http_request(port, &req_sync, &pdf_bytes);
    assert_eq!(status, 200);
    assert!(body.contains("HTTP Async Integration"));

    // 4. POST /convert?async=true (Async)
    let req_async = format!(
        "POST /convert?async=true HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/pdf\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        port, pdf_bytes.len()
    );
    let (status, body) = send_http_request(port, &req_async, &pdf_bytes);
    assert_eq!(status, 202, "Should return 202 Accepted for async task");
    assert!(body.contains("task_id"));
    assert!(body.contains("/status/task_"));

    let json_val: serde_json::Value = serde_json::from_str(&body).unwrap();
    let task_id = json_val["task_id"].as_str().unwrap();

    // 5. Poll GET /status/{task_id}
    let mut task_completed = false;
    for _ in 0..50 {
        let (st, status_body) = send_http_request(
            port,
            &format!(
                "GET /status/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
                task_id, port
            ),
            &[],
        );
        if st == 200 && status_body.contains("\"status\":\"completed\"") {
            assert!(status_body.contains("HTTP Async Integration"));
            task_completed = true;
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        task_completed,
        "Async task should transition to completed state"
    );
}
