use clap::Parser;
use pdf2md_core::{Config, Converter};
use pdf2md_http::HttpServer;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(
    name = "pdf2md-http",
    author = "EcoUni Systems <dev@ecouni.org>",
    version = "0.1.0",
    about = "Lightweight HTTP worker microservice with OpenAPI 3.0 and async task handling"
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
    let config = Config::builder().build();
    let converter = Arc::new(Converter::new(config));

    let server = HttpServer::new(args.host, args.port, converter);
    server.start()
}
