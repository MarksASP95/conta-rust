use crate::{secrets::SPREADSHEETS_BASE_URL, constants::MONTHS};
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
