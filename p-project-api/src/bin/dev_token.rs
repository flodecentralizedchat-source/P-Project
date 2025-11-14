use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    role: Option<String>,
}

fn main() {
    let mut sub = String::from("user");
    let mut hours: i64 = 24;
    let mut role: Option<String> = None;

    for arg in std::env::args().skip(1) {
        if let Some(val) = arg.strip_prefix("sub=") {
            sub = val.to_string();
        } else if let Some(val) = arg.strip_prefix("hours=") {
            hours = val.parse::<i64>().unwrap_or(24);
        } else if let Some(val) = arg.strip_prefix("role=") {
            role = Some(val.to_string());
        }
    }

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = (chrono::Utc::now() + chrono::Duration::hours(hours)).timestamp() as usize;

    let claims = Claims { sub, exp, iat: now, role };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .expect("Failed to encode JWT");
    println!("{}", token);
}

