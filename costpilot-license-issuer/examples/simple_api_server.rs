/// Simple HTTP API Server for License Issuance
///
/// This demonstrates how to use costpilot-license-issuer in a REST API.
/// In production, use a proper framework like Axum, Actix, or Rocket.

use costpilot_license_issuer::{EditionTier, IssuedLicense, LicenseIssuer, LicenseRequest};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

#[derive(Deserialize)]
struct ApiRequest {
    email: String,
    license_key: String,
    edition: String,
    expires_days: i64,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    license: Option<IssuedLicense>,
    error: Option<String>,
}

struct Server {
    issuer: Arc<LicenseIssuer>,
}

impl Server {
    fn new(issuer: LicenseIssuer) -> Self {
        Self {
            issuer: Arc::new(issuer),
        }
    }

    fn handle_request(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        if http_request.is_empty() {
            return;
        }

        let request_line = &http_request[0];

        if request_line.starts_with("POST /api/licenses") {
            // Find Content-Length
            let content_length = http_request
                .iter()
                .find(|line| line.starts_with("Content-Length:"))
                .and_then(|line| line.split(':').nth(1))
                .and_then(|len| len.trim().parse::<usize>().ok())
                .unwrap_or(0);

            // Read body
            let mut body = vec![0u8; content_length];
            if let Ok(_) = stream.as_ref().read_exact(&mut body) {
                if let Ok(api_request) = serde_json::from_slice::<ApiRequest>(&body) {
                    self.handle_issue_license(stream, api_request);
                    return;
                }
            }

            self.send_error_response(stream, "Invalid request body");
        } else if request_line.starts_with("GET /health") {
            self.send_health_response(stream);
        } else {
            self.send_404_response(stream);
        }
    }

    fn handle_issue_license(&self, mut stream: TcpStream, req: ApiRequest) {
        let edition = match req.edition.to_lowercase().as_str() {
            "free" => EditionTier::Free,
            "premium" => EditionTier::Premium,
            "enterprise" => EditionTier::Enterprise,
            _ => {
                self.send_error_response(stream, "Invalid edition tier");
                return;
            }
        };

        let license_request = LicenseRequest {
            email: req.email,
            license_key: req.license_key,
            edition,
            expires_days: req.expires_days,
        };

        match self.issuer.issue_license(license_request) {
            Ok(license) => {
                let response = ApiResponse {
                    success: true,
                    license: Some(license),
                    error: None,
                };
                let json = serde_json::to_string_pretty(&response).unwrap();
                let response = format!(
                    "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    json.len(),
                    json
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                self.send_error_response(stream, &e.to_string());
            }
        }
    }

    fn send_health_response(&self, mut stream: TcpStream) {
        let json = r#"{"status":"ok","service":"costpilot-license-issuer"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            json.len(),
            json
        );
        stream.write_all(response.as_bytes()).unwrap();
    }

    fn send_error_response(&self, mut stream: TcpStream, error: &str) {
        let response = ApiResponse {
            success: false,
            license: None,
            error: Some(error.to_string()),
        };
        let json = serde_json::to_string_pretty(&response).unwrap();
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            json.len(),
            json
        );
        stream.write_all(response.as_bytes()).unwrap();
    }

    fn send_404_response(&self, mut stream: TcpStream) {
        let body = r#"{"error":"Not Found"}"#;
        let response = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CostPilot License API Server ===\n");

    // Load or generate keypair
    let private_key_hex = std::env::var("LICENSE_PRIVATE_KEY").unwrap_or_else(|_| {
        println!("⚠️  LICENSE_PRIVATE_KEY not set, generating new keypair...");
        let keypair = LicenseIssuer::generate_keypair().unwrap();
        println!("   Private Key: {}", hex::encode(&keypair.private_key_bytes));
        println!("   Public Key:  {}", keypair.public_key_hex);
        println!("   Set env: export LICENSE_PRIVATE_KEY={}\n", hex::encode(&keypair.private_key_bytes));
        hex::encode(&keypair.private_key_bytes)
    });

    let private_key_bytes = hex::decode(&private_key_hex)?;
    let issuer = LicenseIssuer::from_private_key_bytes(&private_key_bytes)?;

    println!("✓ License issuer initialized");
    println!("  Fingerprint: {}\n", issuer.fingerprint());

    let server = Server::new(issuer);
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    println!("🚀 Server running on http://127.0.0.1:8080\n");
    println!("Endpoints:");
    println!("  POST /api/licenses  - Issue a license");
    println!("  GET  /health        - Health check\n");
    println!("Example:");
    println!(r#"  curl -X POST http://localhost:8080/api/licenses \"#);
    println!(r#"    -H "Content-Type: application/json" \"#);
    println!(r#"    -d '{{"#);
    println!(r#"      "email": "test@example.com","#);
    println!(r#"      "license_key": "PREMIUM-TEST-123","#);
    println!(r#"      "edition": "premium","#);
    println!(r#"      "expires_days": 365"#);
    println!(r#"    }}'"#);
    println!();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                server.handle_request(stream);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
