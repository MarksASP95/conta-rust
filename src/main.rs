mod secrets;
mod constants;
mod structs;
mod utils;

use inquire::{ Text, Select };
use chrono::{ self, Datelike };
use clap::Command;
use secrets::*;
use constants::*;
use structs::*;
use utils::{token::get_access_token, log::conta_log};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let today = chrono::offset::Local::now();
    let cli_matches = Command::new("conta")
        .about("Interact with Contabilidad personal")
        .subcommand_required(true)
        .subcommand(clap::Command::new("add").about("Add entry"))
        .get_matches();

    let entry: Option<EntryInput> = match cli_matches.subcommand() {
        Some(("add", _)) => {
            let mut amount_usd = Text::new("Amount USD")
                .with_placeholder("skip for Bs")
                .prompt_skippable()
                .unwrap();
            let mut amount_bs: Option<String> = None;
            let mut rate: Option<String> = None;
            if amount_usd == Some("".to_owned()) {
                amount_usd = None;
                amount_bs = Some(Text::new("Amount Bs").prompt().unwrap());
            }
            if amount_bs.is_some() {
                rate = Text::new("Rate").with_placeholder("optionals").prompt_skippable().unwrap();
                if rate == Some("".to_owned()) {
                    rate = None;
                }
            }

            let date = Text::new("Date")
                .with_default(&today.format("%d-%m-%Y").to_string())
                .with_placeholder("(today)")
                .prompt_skippable()
                .unwrap()
                .unwrap();

            let description = Text::new("Description").prompt().unwrap();

            let tag_options: Vec<&str> = vec![
                "activo",
                "aether",
                "carro",
                "casa",
                "comida",
                "deuda",
                "diversion",
                "divisas",
                "estacionamiento",
                "emergencia",
                "misha",
                "odalys",
                "varios",
                "venta"
            ];
            let tag = Select::new("Tag", tag_options).prompt().unwrap().to_owned();

            let form_options: Vec<&str> = vec![
                "banesco",
                "cash",
                "keep",
                "luismi",
                "oficina",
                "paypal",
                "ulises"
            ];
            let form = Select::new("Form", form_options).prompt().unwrap().to_owned();

            let month_index = today.month0();
            let year = today.year();
            Some(EntryInput {
                data: EntryPayload {
                    amount_usd,
                    amount_bs,
                    rate,
                    date,
                    description,
                    form,
                    tag,
                },
                dev_mode: false,
                spr_id: SPREADSHEET_ID.to_owned(),
                sheet_name: [
                    MONTHS.get(month_index as usize)
                        .unwrap()
                        .to_string(),
                    year.to_string(),
                ].join(" "),
            })
        }
        _ => None,
    };

    if entry.is_none() {
        panic!("Could not create entry input");
    }

    let access_token = get_access_token().await.unwrap();

    let client = reqwest::Client::new();
    let add_entry_res = client
        .post(ADD_ENTRY_FN_URL)
        .header("Authorization", ["Bearer", &access_token].join(" "))
        .json(&entry.unwrap())
        .send().await
        .unwrap()
        .json::<EntryResponse>().await
        .unwrap();

    if add_entry_res.success {
        conta_log("Entry saved!");
    } else {
        conta_log(format!("addEntry -> {}", add_entry_res.message.expect("NO MESSAGE")).as_str());
    }

    Ok(())
}
