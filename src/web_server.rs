
#![feature(proc_macro_hygiene, decl_macro)]
use mpsc::SyncSender;
use serde_json::{Result,Value};
use rocket::{error::LaunchError, State};
use std::{self,collections::HashMap, iter::Map, sync::mpsc::{self, Sender}};
use rocket_contrib::json::Json;
use serde::{Deserialize};
use log::info;
use super::TimesDB;
use crossbeam::atomic::AtomicCell;

#[derive(Deserialize)]
struct VDCPTimes{
    times:HashMap<u8,Vec<u16>>
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[put("/times",data="<vdcp_times>")]
fn times(times_db:State<TimesDB>,vdcp_times:Json<VDCPTimes>) -> &'static str {
    for time in &vdcp_times.times{
        let i:usize=time.0.clone().into();
        times_db.0[i].store(time.1.clone());
    }
    info!("get incoming put request: {:?}",vdcp_times.times);
    "set data"
}

pub fn start_server(times_db:  TimesDB ) -> (rocket::Rocket){
    let a=rocket::ignite().mount("/", routes![index,times]).manage(times_db);
   a
     
}