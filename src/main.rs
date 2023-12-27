
use jsonwebtoken::{ Header, Algorithm, EncodingKey, encode };
use secrets::{ADD_ENTRY_FN_URL, SERVICE_ACCOUNT_EMAIL, SPREADSHEET_ID};
use serde::{ Serialize, Deserialize };
use chrono;
use std::fs;
use std::path::Path;

mod secrets;

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
    expires_in: i64,
}

#[derive(Debug, Serialize)]
struct EntryPayload {
    date: String,
    description: String,
    tag: String,
    form: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    rate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "amountUSD"))]
    amount_usd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "amountBs"))]
    amount_bs: Option<String>,
}

#[derive(Debug, Serialize)]
struct EntryInput {
    #[serde(rename(serialize = "sprId"))]
    spr_id: String,
    data: EntryPayload,
    #[serde(rename(serialize = "devMode"))]
    dev_mode: bool,
}
#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenStore {
    token: String,
    expires_at: i64,
}

const SCOPE_CLAIM: &str = "https://www.googleapis.com/auth/drive.readonly";
const TOKEN_REQ_URL: &str = "https://oauth2.googleapis.com/token";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
const CACHE_FILE_PATH: &str = "access_token.json";

fn conta_log(content: &str) {
    println!("conta: {}", content);
}

fn save_token(token: &str, expires_at: i64) -> Result<(), anyhow::Error> {
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
async fn get_access_token() -> Result<String, anyhow::Error> {
    // get from cache
    if Path::new(&CACHE_FILE_PATH).exists() {
        let contents = fs::read_to_string(&CACHE_FILE_PATH).expect("Unable to read file");
        let store = match serde_json::from_str::<AccessTokenStore>(&contents) {
            Ok(store) => { store },
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let access_token = get_access_token().await.unwrap();

    let entry = EntryInput {
        data: EntryPayload {
            amount_usd: Some("24,20".to_owned()),
            amount_bs: None,
            date: "27-12-2023".to_owned(),
            description: "Test from Rust".to_owned(),
            form: "banesco".to_owned(),
            rate: None,
            tag: "varios".to_owned(),
        },
        dev_mode: false,
        spr_id: SPREADSHEET_ID.to_owned(),
    };

    let client = reqwest::Client::new();
    let add_entry_res = client
        .post(ADD_ENTRY_FN_URL)
        .header("Authorization", ["Bearer", &access_token].join(" "))
        .json(&entry)
        .send().await
        .unwrap()
        .text().await
        .unwrap();

    println!("Response: {}", add_entry_res);

    Ok(())
}
