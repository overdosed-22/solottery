#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
use rocket_contrib::json::{Json, JsonValue};
use mysql::*;
use mysql::prelude::*;
use serde::Serialize;
use std::io;
extern crate libc;
use libc::uint32_t;
extern crate chrono;
use chrono::{NaiveDate, NaiveDateTime};

#[derive(Serialize,Deserialize)]
struct PageRequest {
    page_index: u16,
    page_size: u16,
    address: String,
    ticket_id: u32
}

#[derive(Serialize,Deserialize)]
struct ticket {
    id:u32,
    ticket_id: u32,
    ticket_number: String,
    ticket_letter: String,
    address: String,
    is_win: u8,
    is_exchange: u8,
    mint_time: Option<chrono::NaiveDateTime>
}

#[get("/",format = "json",data = "<page_request>")]
//建立连接
fn get_tickets(page_request: Json<PageRequest>) -> String {
    let url = "mysql://pete:rbd2529421840@144.91.79.134:3306/solottery";
    let opts = Opts::from_url(url).unwrap();
    let pool = Pool::new(opts).unwrap();
    let mut conn = pool.get_conn().unwrap();
    if page_request.address.is_empty(){
        //查询所有ticket
        let res = conn.query_map("select * from ticket",|(id,ticket_id,ticket_number,ticket_letter,address,is_win,is_exchange,mint_time)| ticket{
            id:id,
            ticket_id:ticket_id,
            ticket_number:ticket_number,
            ticket_letter:ticket_letter,
            address:address,
            is_win:is_win,
            is_exchange:is_exchange,
            mint_time:mint_time,
        },
        ).expect("Query failed.");
        serde_json::to_string(&res).unwrap()
    }else{
        //查询我的ticket
        let res = conn.exec_first("select * from ticket where ticket_id = :ticket_id",params! {
            "ticket_id" => page_request.ticket_id
        })
        .map(
            |row| {
                row.map(|(id,ticket_id,ticket_number,ticket_letter,address,is_win,is_exchange,mint_time)| ticket{
                    id:id,
                    ticket_id:ticket_id,
                    ticket_number:ticket_number,
                    ticket_letter:ticket_letter,
                    address:address,
                    is_win:is_win,
                    is_exchange:is_exchange,
                    mint_time:mint_time,
                })
            },
        ).expect("Query failed");
        serde_json::to_string(&res).unwrap()
    }
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

fn main(){
    rocket().launch();
}