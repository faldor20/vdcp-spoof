
#![feature(proc_macro_hygiene, decl_macro)]
use mpsc::SyncSender;
use serde_json::{Result,Value};
use rocket::{error::LaunchError, State};
use std::{self,collections::HashMap, iter::Map, sync::mpsc::{self, Sender}};
use rocket_contrib::json::Json;
use serde::{Deserialize};



#[derive(Deserialize)]
struct VDCPTimes{
    times:HashMap<u8,Vec<u16>>
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[put("/times",data="<vdcp_times>")]
fn times(tx:State<SyncSender<(u8,Vec<u64>)>>,vdcp_times:Json<VDCPTimes>) -> &'static str {
    ""
}

pub fn start_server(tx:SyncSender<(u8,Vec<u64>)>) ->LaunchError{
     rocket::ignite().mount("/", routes![index,times]).launch()
     
}