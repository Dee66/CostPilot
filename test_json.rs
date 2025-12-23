use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub email: String,
    pub expires: String,
    pub signature: String,
}

fn main() {
    let json = r#"{"key": "test-key", "email": "test@example.com"}"#;
    let result: Result<License, _> = serde_json::from_str(json);
    println!("Result: {:?}", result);
}
