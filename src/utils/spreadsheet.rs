use crate::{
    secrets::{ SPREADSHEETS_BASE_URL, FNS_WEB_APP_BASE_URL, FN_GET_STATUS, FN_ADD_ENTRIES },
    constants::MONTHS,
};
use chrono::{ self, Datelike };

pub fn get_spreadsheet_read_url(spr_id: &str, range: &str) -> String {
    return format!("{}/{}/values/{}", SPREADSHEETS_BASE_URL, spr_id, range);
}

pub fn get_latest_sheet_name() -> String {
    let today = chrono::offset::Local::now();
    let month_index = today.month0();
    let year = today.year();
    return [
        MONTHS.get(month_index as usize)
            .unwrap()
            .to_string(),
        year.to_string(),
    ].join(" ");
}

pub enum ContaFunctionName {
    AddEntries,
    GetStatus,
}
pub fn get_fn_url(fn_name: ContaFunctionName) -> String {
    match fn_name {
        ContaFunctionName::AddEntries => {
            return format!("{}?functionName={}", FNS_WEB_APP_BASE_URL, FN_ADD_ENTRIES);
        }
        ContaFunctionName::GetStatus => {
            return format!("{}?functionName={}", FNS_WEB_APP_BASE_URL, FN_GET_STATUS);
        }
    }
}
