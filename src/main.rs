#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
use crossbeam::atomic::AtomicCell;
use env::args;
use std::{
    env, 
    
    thread,
};
mod vdcp;
use flexi_logger::*;
use log::*;
mod serial;
mod web_server;
pub struct TimesDB(Vec<AtomicCell<Vec<u16>>>);

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
    setup_logging();
    let a: Vec<String> = args().collect();
    info!("got {:?} args", a);
    //This vector stores all the times and is written to by the webserver and read from by the vdcp
    //4 segements
    //one vec is created per port name
    let vector=(0..a.len()-1).map(|_|AtomicCell::new(vec![0u16;4]) ).collect::<Vec<AtomicCell<Vec<u16>>>>();
    let times_db:TimesDB = TimesDB(vector);

    let rocket_server=web_server::start_server( times_db);
    //We now need a refernce to the times_db given to the webserver
    let times_db=rocket_server.state::<TimesDB>().expect("webserver did not return times-db state, cannot contuine without timesdb");

    crossbeam::thread::scope(|s| {  
        match a.len() {
            x if x > 2 => {
                //skips the first irrelivant arg and iterates over them giving each serial reader its own port and id
                for i in 1..a.len() {
                    let com=&a[i];
                    s.spawn(move |_|{ serial::start(com, &times_db.0[i-1])
                        .expect("Completly failed interacting with serial port")});
                    }
                }
                
                _ => info!("Command should be:'VDCP 'Port name1' 'portname2' 'portname3' etc etc'"),
                
            }
        }).expect("Failed running serial threads");

    info!("finish");
}
