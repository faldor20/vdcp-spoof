
#![feature(proc_macro_hygiene, decl_macro)]
use mpsc::SyncSender;

use rocket::{error::LaunchError, State};
use rocket_cors::CorsOptions;
use std::{self, collections::HashMap,  sync::{Mutex, mpsc::{self, Sender}}};
use rocket_contrib::json::Json;
use serde::{Deserialize,Serialize};
use log::info;
use super::config::Config;


#[derive(Deserialize,Serialize)]
struct VDCPTimes{
   pub times:HashMap<u8,Vec<u16>>
}

 pub type TimesUpdaters=Vec<SyncSender<Vec<u16>>>;
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[put("/times",data="<vdcp_times>")]
fn times(times_db:State< TimesUpdaters>,vdcp_times:Json<VDCPTimes>) -> &'static str {
    for time in &vdcp_times.times{
        let i:usize=time.0.clone().into();
        
        times_db[i].send(time.1.clone());
    }
    info!("got sent times from website. Times: {:?}",vdcp_times.times);
    "set data"
}
#[get("/ports")]
fn ports(conf:State<Config>) -> Json<Config> {

    info_!("got request for ports");
    Json(conf.clone())
}

pub fn start_server(config:Config,times_db: TimesUpdaters ) -> (rocket::Rocket){
    let mut times= VDCPTimes{times:HashMap::new()};
    times.times.insert(1, vec![33u16;3]);
    let data=serde_json::to_string_pretty(&times).expect("failed serializing test");
    info!("data {:}",data);
    let cors_opts=CorsOptions{..Default::default()}.to_cors().expect("failed making cors options");
    let a=rocket::ignite().mount("/", routes![index,times,ports]).manage(times_db).manage(config).attach(cors_opts);
   a
     
}