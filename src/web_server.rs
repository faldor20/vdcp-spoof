
#![feature(proc_macro_hygiene, decl_macro)]
use serde_json::{Result,Value};
use rocket::error::LaunchError;
use std::{collections::HashMap, iter::Map, sync::mpsc::{self, Sender}};
use rocket_contrib::json::Json;
use serde::{Deserialize};



#[derive(Deserialize)]
struct VDCPTimes{
    times:HashMap<u8,Vec<u64>>
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[put("/times",data="<times>")]
fn set_times(times:Json<VDCPTimes>) -> &'static str {
    "Hello, world!"
}

pub fn start_server(tx:Sender<(u8,Vec<u64>)>) ->LaunchError{
     rocket::ignite().mount("/", routes![index]).launch()
     
}