// src/bin/serve.rs

use std::fs::File;
use std::io::Read;
use std::path::Path;
use tiny_http::{Header, Request, Response, Server, StatusCode};

/// Guess a MIME type from the file extension
fn mime_from_path(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") || path.ends_with(".mjs") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else {
        "application/octet-stream"
    }
}

/// Handle each incoming HTTP request
fn handle_request(request: Request) {
    let url = request.url();

    // Map "/" → "index.html", else strip leading "/"
    let path = if url == "/" {
        "index.html".to_string()
    } else {
        url.trim_start_matches('/').to_string()
    };

    println!("Request for: {}", path);

    let response = if Path::new(&path).exists() {
        match File::open(&path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                if let Err(e) = file.read_to_end(&mut buf) {
                    eprintln!("Error reading {}: {}", path, e);
                    Response::from_data(Vec::new()).with_status_code(StatusCode(500))
                } else {
                    let mime = mime_from_path(&path);
                    let mut resp = Response::from_data(buf);

                    // Add headers
                    resp.add_header(
                        Header::from_bytes("Content-Type", mime)
                            .expect("failed to create Content-Type header"),
                    );

                    // Add CORS headers for WASM
                    resp.add_header(
                        Header::from_bytes("Access-Control-Allow-Origin", "*")
                            .expect("failed to create CORS header"),
                    );

                    // Cache control for development
                    resp.add_header(
                        Header::from_bytes("Cache-Control", "no-cache, no-store, must-revalidate")
                            .expect("failed to create Cache-Control header"),
                    );

                    resp
                }
            }
            Err(e) => {
                eprintln!("Failed to open {}: {}", path, e);
                Response::from_data(Vec::new()).with_status_code(StatusCode(500))
            }
        }
    } else {
        eprintln!("File not found: {}", path);
        Response::from_data(Vec::new()).with_status_code(StatusCode(404))
    };

    if let Err(e) = request.respond(response) {
        eprintln!("Failed to send response: {}", e);
    }
}

fn main() {
    let server = Server::http("127.0.0.1:8000").unwrap();
    println!("🚀 Serving Scout Pathfinder on http://127.0.0.1:8000");
    println!("📁 Serving files from current directory");
    println!("Press Ctrl+C to stop the server\n");

    for request in server.incoming_requests() {
        handle_request(request);
    }
}
