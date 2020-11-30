#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
use env::args;
use std::{env, io, sync::mpsc::{self, Receiver}, thread};
mod vdcp;
use flexi_logger::*;
use log::*;
mod web_server;
mod serial;
fn setup_logging() {
    let res = Logger::with_str("info")
        .log_target(LogTarget::File).directory("./Logs/")
        .duplicate_to_stdout(Duplicate::All) // write logs to file
        .duplicate_to_stderr(Duplicate::Warn) // print warnings and errors also to the console
        .start();
    match res {
        Err(e) => println!("failed to init logging. Reason: {0}", e),
        Ok(_) => (),
    }
}
fn setup_webserver()->Receiver<(u8,Vec<u64>)>{
    let (tx,rx)=mpsc::channel();
    thread::spawn(move||web_server::start_server(tx));
    rx
}
fn main() {
    setup_logging();
    let receiver=setup_webserver();
    let a: Vec<String> = args().collect();
    info!("got {:?} args", a);

    match a.len() {
        2 => serial::start(&a[1]).expect("Completly failed interacting with serial port"),
        _ => info!("command should be:'VDCP 'Port name''"),
    }
    
    info!("finish");
}