use std::{ path::Path, fs, include_bytes };
use jsonwebtoken::{ EncodingKey, Header, Algorithm, encode };
use crate::{
    constants::{ CACHE_FILE_PATH, SCOPE_CLAIM, TOKEN_REQ_URL, GRANT_TYPE },
    structs::{ AccessTokenStore, Claims, AccessTokenRequestBody, AccessTokenResponseBody },
    secrets::SERVICE_ACCOUNT_EMAIL,
};

use super::log::conta_log;

pub fn save_token(token: &str, expires_at: i64) -> Result<(), anyhow::Error> {
    if Path::new(&CACHE_FILE_PATH).exists() {
        fs::remove_file(CACHE_FILE_PATH).unwrap();
    }
    let store = AccessTokenStore {
        expires_at,
        token: token.to_owned(),
    };
    let mut contents = serde_json::to_string(&store).unwrap();
    fs::write(CACHE_FILE_PATH, &mut contents)?;
    Ok(())
}

pub async fn get_access_token() -> Result<String, anyhow::Error> {
    // get from cache
    if Path::new(&CACHE_FILE_PATH).exists() {
        let contents = fs::read_to_string(&CACHE_FILE_PATH).expect("Unable to read file");
        let store = match serde_json::from_str::<AccessTokenStore>(&contents) {
            Ok(store) => { store }
            Err(_) => {
                conta_log("warning: token cache is malformed. ignoring");
                AccessTokenStore { token: "".to_owned(), expires_at: 0 }
            }
        };

        let current_time_secs = chrono::offset::Local::now().timestamp_millis() / 1000;
        if current_time_secs < store.expires_at {
            conta_log("token recovered from cache");
            return Ok(store.token);
        }
    }

    // if cache miss, get new
    conta_log("cache miss. getting new token...");
    let service_account_email = SERVICE_ACCOUNT_EMAIL;
    let key = EncodingKey::from_rsa_pem(include_bytes!("../private_key")).unwrap();

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

    let client = reqwest::Client::new();

    let res = client
        .post(TOKEN_REQ_URL)
        .json(&req_body)
        .send().await
        .unwrap()
        .json::<AccessTokenResponseBody>().await
        .unwrap();

    conta_log("got fresh token");

    match save_token(&res.access_token, expire_time_secs) {
        Ok(_) => {
            conta_log("token saved");
        }
        _ => {
            conta_log("warning -> token could not be saved");
        }
    }

    Ok(res.access_token)
}
