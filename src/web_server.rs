#![feature(proc_macro_hygiene, decl_macro)]
use mpsc::SyncSender;

use super::config::Config;
use log::{error, info};
use rocket::{State, response::NamedFile};
use rocket_contrib::json::Json;
use rocket_cors::CorsOptions;
use serde::{Deserialize, Serialize};
use std::{self, collections::HashMap, io, path::{Path, PathBuf}, sync::mpsc::{self}};

#[derive(Deserialize, Serialize)]
struct VDCPTimes {
    pub times: HashMap<u8, Vec<u16>>,
}

pub type TimesUpdaters = Vec<SyncSender<Vec<u16>>>;

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).ok()
}
#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open("public/index.html")
}
#[put("/api/times", data = "<vdcp_times>")]
fn times(times_db: State<TimesUpdaters>, vdcp_times: Json<VDCPTimes>) -> &'static str {
    for time in &vdcp_times.times {
        let i: usize = time.0.clone().into();

        match times_db[i].send(time.1.clone()) {
            Err(e) => {
                error!("Failed sending times to thread {:}", e)
            }
            _ => (),
        }
    }
    info!("got sent times from website. Times: {:?}", vdcp_times.times);
    "set data"
}
#[get("/api/ports")]
fn ports(conf: State<Config>) -> Json<Config> {
    info_!("got request for ports");
    Json(conf.clone())
}

pub fn start_server(config: Config, times_db: TimesUpdaters) -> rocket::Rocket {
    let mut times = VDCPTimes {
        times: HashMap::new(),
    };
    times.times.insert(1, vec![33u16; 3]);
    let data = serde_json::to_string_pretty(&times).expect("failed serializing test");
    info!("data {:}", data);
    let cors_opts = CorsOptions {
        allowed_origins:rocket_cors::AllOrSome::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("failed making cors options");
    let a = rocket::ignite()
        .mount("/", routes![index, times, ports,files])
        .manage(times_db)
        .manage(config)
        .attach(cors_opts);
    a
}
