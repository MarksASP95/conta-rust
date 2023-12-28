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
use utils::{ token::get_access_token, log::conta_log };
#[macro_use] extern crate prettytable;
use prettytable::Table;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let today = chrono::offset::Local::now();
    let cli_matches = Command::new("conta")
        .about("Interact with Contabilidad personal")
        .subcommand_required(true)
        .subcommand(clap::Command::new("add").about("Add entry"))
        .get_matches();

    let mut keep_adding = true;
    let mut entries = Vec::<EntryPayload>::new();
    while keep_adding {
        let entry: Option<EntryPayload> = match cli_matches.subcommand() {
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
                    rate = Text::new("Rate")
                        .with_placeholder("optional")
                        .prompt_skippable()
                        .unwrap();
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
                    "farmacia",
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

                Some(EntryPayload {
                    amount_usd,
                    amount_bs,
                    rate,
                    date,
                    description,
                    form,
                    tag,
                })
            }
            _ => None,
        };

        if entry.is_none() {
            panic!("Could not create entry input");
        }

        entries.push(entry.unwrap());

        println!("---------------------------------------------");

        let add_another_str = Text::new("Add another entry?")
            .with_default("y")
            .with_placeholder("[Y/n]")
            .prompt()
            .unwrap()
            .to_lowercase();

        println!("---------------------------------------------");

        if add_another_str != "y" {
            keep_adding = false;
        }
    }

    let month_index = today.month0();
    let year = today.year();
    let entries_input = EntriesInput {
        entries: &entries,
        dev_mode: false,
        spr_id: SPREADSHEET_ID.to_owned(),
        sheet_name: [
            MONTHS.get(month_index as usize)
                .unwrap()
                .to_string(),
            year.to_string(),
        ].join(" "),
    };

    // Create the table
    let mut table = Table::new();

    // Add a row per time
    table.add_row(row!["Rate", "AmountBs", "Amount USD", "Description", "Date", "Tag", "Form"]);
    for entry in &entries {
        table.add_row(row![
            entry.rate.clone().unwrap_or("".to_owned()), 
            entry.amount_bs.clone().unwrap_or("".to_owned()), 
            entry.amount_usd.clone().unwrap_or("(calculated)".to_owned()),
            entry.description,
            entry.date,
            entry.tag,
            entry.form,
        ]);
    }

    table.printstd();

    let confirm = Text::new("Confirm?")
        .with_default("y")
        .with_placeholder("[Y/n (exit)]")
        .prompt()
        .unwrap()
        .to_lowercase();

    if confirm != "y" {
        return Ok(());
    }

    let access_token = get_access_token().await.unwrap();

    let client = reqwest::Client::new();
    let add_entry_res = client
        .post(ADD_ENTRY_FN_URL)
        .header("Authorization", ["Bearer", &access_token].join(" "))
        .json(&entries_input)
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
