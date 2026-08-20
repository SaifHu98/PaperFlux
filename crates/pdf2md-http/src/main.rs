use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use clap::Parser;
use pdf2md_core::{Config, Converter};

#[derive(Parser, Debug)]
#[command(
    name = "pdf2md-http",
    author = "EcoUni Systems <dev@ecouni.org>",
    version = "0.1.0",
    about = "Lightweight HTTP worker server for remote PDF to Markdown conversion"
)]
struct Args {
    /// Host address to bind
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let bind_addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&bind_addr)?;
    println!("pdf2md HTTP microservice listening on http://{}", bind_addr);

    let config = Config::builder().build();
    let converter = Arc::new(Converter::new(config));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let conv = Arc::clone(&converter);
                std::thread::spawn(move || {
                    let _ = handle_client(stream, conv);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, converter: Arc<Converter>) -> std::io::Result<()> {
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

        if first_line.starts_with("GET /health") {
            let body = r#"{"status":"ok","version":"0.1.0"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes())?;
            return Ok(());
        }

        if first_line.starts_with("POST /convert") {
            let body_bytes = &buffer[h_end..h_end + content_length];
            match converter.convert_bytes(body_bytes) {
                Ok(result) => {
                    let json_response = serde_json::json!({
                        "markdown": result.markdown,
                        "diagnostics": result.diagnostics
                    });
                    let json_str = json_response.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        json_str.len(),
                        json_str
                    );
                    stream.write_all(response.as_bytes())?;
                    return Ok(());
                }
                Err(e) => {
                    let err_body = serde_json::json!({
                        "error": e.to_string()
                    }).to_string();
                    let response = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        err_body.len(),
                        err_body
                    );
                    stream.write_all(response.as_bytes())?;
                    return Ok(());
                }
            }
        }
    }

    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    stream.write_all(not_found.as_bytes())?;
    Ok(())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}
