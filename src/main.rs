#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use std::thread;
mod vdcp;
use flexi_logger::*;
use log::*;
mod config;
mod serial;
mod web_server;
use vdcp::types::{PortConfig, PortStatus};
fn setup_logging() {
    let res = Logger::with_str("info")
        .log_target(LogTarget::File)
        .directory("./Logs/")
        .duplicate_to_stdout(Duplicate::All) // write logs to file
        .duplicate_to_stderr(Duplicate::Warn) // print warnings and errors also to the console
        .start();
    match res {
        Err(e) => println!("failed to init logging. Reason: {0}", e),
        Ok(_) => (),
    }
}

fn main() {
    let conf: config::Config = confy::load_path("./config.yaml").unwrap();
    setup_logging();

    info!("got {:?} config", conf);
    //This vector stores all the times and is written to by the webserver and read from by the vdcp
    //4 segements
    //one vec is created per port name
    let (senders, mut receivers): (Vec<_>, Vec<_>) = (0..conf.ports.len())
        .map(|_| std::sync::mpsc::sync_channel::<Vec<u16>>(100))
        .unzip();
    let rocket_server = web_server::start_server(conf.clone(), senders);

    //We now need a refernce to the times_db given to the webserver

    let threads: Vec<_> = receivers
        .drain(..)
        .zip(conf.ports)
        .map(|(rec, port)| {
            thread::spawn(move || {
                info!("spawing port monitoring thread");
                
                let config=PortConfig{number:port.number,port_status:PortStatus::Idle};
                serial::start(port.port, rec,config)
                    .expect("Completly failed interacting with serial port")
            })
        })
        .collect();

    //skips the first irrelivant arg and iterates over them giving each serial reader its own port and id
    /* for (i,recv) in receivers.drain(..).enumerate() {
    //let com=&a[i];
    let a=a[i].clone();
    thread::spawn(move ||{ serial::start(a, recv)
        .expect("Completly failed interacting with serial port")});
    } */

    rocket_server.launch();
    for thread in threads {
        match thread.join() {
            Err(e) => {
                error!("thread erroed with {:?}", e)
            }
            _ => (),
        }
    }

    /* crossbeam::thread::scope(|s| {

    let rocket_server=web_server::start_server(senders);
    // let times_db=rocket_server.state::<TimesDB>().expect("webserver did not return times-db state, cannot contuine without timesdb");
    rocket_server.launch();
    match a.len() {
        x if x > 2 => {
            //skips the first irrelivant arg and iterates over them giving each serial reader its own port and id
            for i in 1..a.len() {
                let com=&a[i];

                s.spawn( |_|{ serial::start(com, receivers[i])
                    .expect("Completly failed interacting with serial port")});
                }
            }

            _ => info!("Command should be:'VDCP 'Port name1' 'portname2' 'portname3' etc etc'"),

        }

    }).expect("Failed running serial threads"); */

    info!("finish");
}
