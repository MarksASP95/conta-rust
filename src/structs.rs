use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub scope: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenRequestBody {
    pub grant_type: String,
    pub assertion: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResponseBody {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct EntryPayload {
    pub date: String,
    pub description: String,
    pub tag: String,
    pub form: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "amountUSD"))]
    pub amount_usd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename(serialize = "amountBs"))]
    pub amount_bs: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EntriesInput<'a> {
    #[serde(rename(serialize = "sprId"))]
    pub spr_id: String,
    #[serde(rename(serialize = "sheetName"))]
    pub sheet_name: String,
    pub entries: &'a Vec<EntryPayload>,
    #[serde(rename(serialize = "devMode"))]
    pub dev_mode: bool,
}

#[derive(Debug, Deserialize)]
pub struct EntryResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenStore {
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct RangeValuesResponse {
    pub range: String,
    #[serde(rename = "majorDimension")]
    pub major_dimension: String,
    pub values: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ContaStatusInput {
    #[serde(rename(serialize = "sprId"))]
    pub spr_id: String,
    #[serde(rename(serialize = "sheetName"))]
    pub sheet_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ContaStatusPayload {
    pub general: Vec<Vec<String>>,
    pub distribution: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ContaStatusResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: ContaStatusPayload,
}