use jsonwebtoken::{ Header, Algorithm, EncodingKey, encode };
use serde::{ Serialize, Deserialize };
use chrono;
use securestore::{KeySource, SecretsManager};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenRequestBody {
    grant_type: String,
    assertion: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenResponseBody {
    access_token: String,
    expires_in: i16,
}

const SCOPE_CLAIM: &str = "https://www.googleapis.com/auth/drive.file";
const TOKEN_REQ_URL: &str = "https://oauth2.googleapis.com/token";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
#[tokio::main]
async fn main() {
    let key_path = Path::new("secrets.key");
    let sman = SecretsManager::load("secrets.json", KeySource::Path(key_path)).unwrap();
    let service_account_email: String = sman.get("service_account_email").unwrap();

    let key = EncodingKey::from_rsa_pem(include_bytes!("private_key")).unwrap();

    let current_time_secs = chrono::offset::Local::now().timestamp_millis() / 1000;
    let expire_time_secs = current_time_secs + 3600;
    let claims = Claims {
        iss: String::from(service_account_email),
        scope: String::from(SCOPE_CLAIM),
        aud: String::from(TOKEN_REQ_URL),
        iat: current_time_secs,
        exp: expire_time_secs,
    };

    let mut header = Header::default();
    header.alg = Algorithm::RS256;
    header.typ = Option::Some("JWT".to_owned());

    let token = encode(&header, &claims, &key).unwrap();

    let req_body = AccessTokenRequestBody {
        grant_type: GRANT_TYPE.to_owned(),
        assertion: token,
    };

    let res = reqwest::Client
        ::new()
        .post(TOKEN_REQ_URL)
        .json(&req_body)
        .send().await
        .unwrap()
        .json::<AccessTokenResponseBody>().await
        .unwrap();

    println!("{:?}", res);
}
