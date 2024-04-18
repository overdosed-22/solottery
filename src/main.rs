#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
use rocket_contrib::json::{Json, JsonValue};
use mysql::*;
use mysql::prelude::*;
use chrono::{NaiveDateTime};
use std::fs;
use std::io::{self, ErrorKind};
use std::error::Error;
use std::result::Result as StdResult;  // 引入标准库的Result并重命名，避免冲突


#[derive(Serialize, Deserialize)]
struct PageRequest {
    page_index: u16,
    page_size: u16,
    address: String,
    ticket_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticket {
    id: u32,
    ticket_id: u32,
    ticket_number: String,
    ticket_letter: String,
    address: String,
    is_win: u8,
    is_exchange: u8,
    mint_time: Option<NaiveDateTime>,
}

#[get("/", format = "json", data = "<page_request>")]
fn get_tickets(page_request: Json<PageRequest>) -> StdResult<JsonValue, Box<dyn Error>> {
    let config_path = "Settings.ini";
    let url = get_database_url(config_path, "database_url")?;
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let mut query = "SELECT * FROM ticket".to_string();

    // Initialize parameters with potential values
    let mut params = if !page_request.address.is_empty() {
        query += " WHERE address = :address AND ticket_id >= :start_id";
        params! {
            "address" => &page_request.address,
            "start_id" => page_request.ticket_id,
            "limit" => page_request.page_size as i64,  // Casting to i64 might be necessary depending on your SQL driver
            "offset" => (page_request.page_index as i64 - 1) * page_request.page_size as i64,
        }
    } else {
        query += " WHERE ticket_id >= :start_id";
        params! {
            "start_id" => page_request.ticket_id,
            "limit" => page_request.page_size as i64,
            "offset" => (page_request.page_index as i64 - 1) * page_request.page_size as i64,
        }
    };

    query += " ORDER BY ticket_id ASC LIMIT :limit OFFSET :offset";

    match conn.exec_map(
        &query,
        params,
        |(id, ticket_id, ticket_number, ticket_letter, address, is_win, is_exchange, mint_time): (
            u32,
            u32,
            String,
            String,
            String,
            u8,
            u8,
            Option<NaiveDateTime>,
        )| Ticket {
            id,
            ticket_id,
            ticket_number,
            ticket_letter,
            address,
            is_win,
            is_exchange,
            mint_time,
        },
    ) {
        Ok(tickets) => Ok(json!({ "tickets": tickets })),
        Err(_) => Err(Box::new(io::Error::new(ErrorKind::Other, "Failed to fetch tickets"))),
    }
}

fn get_database_url(config_path: &str, key: &str) -> StdResult<String, Box<dyn Error>> {
    let contents = fs::read_to_string(config_path)?;
    contents
        .lines()
        .find_map(|line| {
            let mut parts = line.splitn(2, '=');
            let current_key = parts.next()?.trim();
            let value = parts.next()?.trim();
            if current_key == key {
                Some(value.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| From::from(format!("Key `{}` not found in config", key)))
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
    .mount("/ticket",routes![get_tickets])
    .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}
