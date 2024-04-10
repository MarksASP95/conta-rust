mod secrets;
mod constants;
mod structs;
mod utils;

use inquire::{ Text, Select };
use clap::Command;
use secrets::*;
use structs::*;
use utils::{ token::get_access_token, log::conta_log };
#[macro_use]
extern crate prettytable;
use prettytable::{ Table, Cell, Row };
use crate::utils::spreadsheet::*;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let today = chrono::offset::Local::now();
    let cli_matches = Command::new("conta")
        .about("Interact with Contabilidad personal")
        .subcommand_required(true)
        .subcommand(clap::Command::new("add").about("Add entries"))
        .subcommand(clap::Command::new("status").about("Check how much money you have"))
        .get_matches();

    match cli_matches.subcommand() {
        Some(("add", _)) => {
            let mut entries = Vec::<EntryPayload>::new();
            let mut keep_adding = true;
            while keep_adding {
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
                    "medico",
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

                let entry = Some(EntryPayload {
                    amount_usd,
                    amount_bs,
                    rate,
                    date,
                    description,
                    form,
                    tag,
                });

                if entry.is_none() {
                    panic!("Could not create entry input");
                }

                entries.push(entry.unwrap());

                println!("---------------------------------------------");

                let add_another_str = Text::new("Add another entry?")
                    .with_default("n")
                    .with_placeholder("[y/N]")
                    .prompt()
                    .unwrap()
                    .to_lowercase();

                println!("---------------------------------------------");

                if add_another_str == "y" {
                    continue;
                } else {
                    keep_adding = false;
                }

                let entries_input = EntriesInput {
                    entries: &entries,
                    dev_mode: false,
                    spr_id: SPREADSHEET_ID.to_owned(),
                    sheet_name: get_latest_sheet_name(),
                };

                let mut table = Table::new();

                table.add_row(
                    row!["Rate", "AmountBs", "Amount USD", "Description", "Date", "Tag", "Form"]
                );
                for entry in &entries {
                    table.add_row(
                        row![
                            entry.rate.clone().unwrap_or("".to_owned()),
                            entry.amount_bs.clone().unwrap_or("".to_owned()),
                            entry.amount_usd.clone().unwrap_or("(calculated)".to_owned()),
                            entry.description,
                            entry.date,
                            entry.tag,
                            entry.form
                        ]
                    );
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
                    .post(get_fn_url(ContaFunctionName::AddEntries))
                    .header("Authorization", ["Bearer", &access_token].join(" "))
                    .json(&entries_input)
                    .send().await
                    .unwrap()
                    .json::<EntryResponse>().await
                    .unwrap();

                if add_entry_res.success {
                    conta_log("Entry saved!");
                } else {
                    conta_log(
                        format!(
                            "addEntry -> {}",
                            add_entry_res.message.expect("NO MESSAGE")
                        ).as_str()
                    );
                }
            }
        }
        Some(("status", _)) => {
            let client = reqwest::Client::new();
            let access_token = get_access_token().await.unwrap();
            let input = ContaStatusInput {
                sheet_name: get_latest_sheet_name(),
                spr_id: SPREADSHEET_ID.to_owned(),
            };
            let status_res = client
                .post(get_fn_url(ContaFunctionName::GetStatus))
                .json(&input)
                .header("Authorization", ["Bearer", &access_token].join(" "))
                .send().await
                .unwrap()
                .json::<ContaStatusResponse>().await
                .unwrap();

            let mut general_table = Table::new();
            {
                let header_cells = status_res.data.general[0]
                    .iter()
                    .map(|text| Cell::new(&text))
                    .collect();

                let amounts_cells = status_res.data.general[1]
                    .iter()
                    .map(|text| Cell::new(&text))
                    .collect();
                general_table.add_row(Row::new(header_cells));
                general_table.add_row(Row::new(amounts_cells));
            }

            let mut distribution_table = Table::new();
            {
                let header_cells = status_res.data.distribution[0]
                    .iter()
                    .map(|text| Cell::new(&text))
                    .collect();

                let amounts_cells = status_res.data.distribution[1]
                    .iter()
                    .map(|text| Cell::new(&text))
                    .collect();
                distribution_table.add_row(Row::new(header_cells));
                distribution_table.add_row(Row::new(amounts_cells));
            }

            println!("\nGENERAL");
            general_table.printstd();

            println!("\nDISTRIBUTION");
            distribution_table.printstd();
        }
        _ => (),
    }

    Ok(())
}
