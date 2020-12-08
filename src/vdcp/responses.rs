use std::{error::Error, str::from_utf8};

use super::types::*;
use log::*;
fn simp(data: Vec<u8>) -> Response {
    Response::Simple(data)
}
fn msg(data: Vec<u8>) -> Response {
    Response::Message(data)
}

fn play(message: &Message, _: &Vec<u16>, config: &mut PortConfig) -> Response {
    info!("Playing port");
    config.port_status = PortStatus::Playing;
    simp(vec![0x04])
}
fn active_id(message: &Message, _: &Vec<u16>, config: &mut PortConfig) -> Response {
    match config.port_status {
        PortStatus::Idle => return msg(vec![0x0]),
        _ => {
            let mut prefix = vec![0x1];
            prefix.append(&mut config.get_cued_clip().clone());
            msg(prefix)
        }
    }
}
fn stop(message: &Message, _: &Vec<u16>, config: &mut PortConfig) -> Response {
    config.port_status = PortStatus::Idle;
    config.next_clip();
    simp(vec![0x04])
}
fn size_request(message: &Message, clip_times: &Vec<u16>, config: &mut PortConfig) -> Response {
    let clip_name = from_utf8(&message.data).unwrap_or("failed to convert from bytes to utf8");
    info!("size requested for clip {:?}", clip_name);
    let stuff = || -> Result<Response, Box<dyn Error>> {
        //the last data byte should tell us the clip number as a utf8 byte
        let last = message.data.last().ok_or("data was empty")?;

        //if we take a utf8 number char byte and subtract hex 30 from it it becomes the number the charctor is
        let index: usize = (last - 0x30) as usize;

        let a = clip_times
            .get(index - 1)
            .ok_or("clip time requested didn't exist")?;
        //This gets our hours and then seconds.
        let minutes = a / 60u16;
        let seconds = a - (minutes * 60u16);
        info!("clip {:} is {:}:{:}", clip_name, minutes, seconds);
        //data is: frames|seconds|minutes|hours
        Ok(msg(vec![0x00, seconds as u8, minutes as u8, 0x00]))
    };

    stuff().unwrap_or_else(|err: Box<dyn Error>| {
        error!("Failed processing size request. Reason: {:?}", err);
        msg(vec![0x0, 0, 0, 0])
    })
}
pub fn unknown_command(msg: &Message) -> Response {
    unsafe {
        warn!(
            "(hex)received unknown command|{:x?}|{:x?}|{:x?}|{:x?}|{:x?}|",
            msg.byte_count, msg.command1.byte, msg.command_code, msg.data, msg.checksum
        );
    }
    simp(vec![0x05, 0x0])
}
pub fn get_commands() -> Vec<Command> {
    let size_request: Command = Command::new("size_request", 0xb, 0x14, size_request);
    let system_status: Command = Command::new("system_status", 0x3, 0x10, |_, _, conf| {
        msg(vec![0x02, 0x00, conf.clip_status.clone() as u8])
    }); //?NOTE: The return here is the number of ids stored by the vdcp server. i think it can remain constant and simply be the max number of clips we ever have

    let open_port: Command = Command::new("open_port", 0x3, 0x01, |_, _, _| (msg(vec![0x01]))); // opened:01 denied:00
    let close_port: Command = Command::new("close_port", 0x2, 0x21, |_, _, _| (simp(vec![0x04]))); // opened:01 denied:00

    let port_status: Command = Command::new("port_status", 0x3, 0x05, |_, _, config| {
        // status that may be of use:
        //vec![0x5, 0x0, 0x0, 0x0, 0x0, 0x80]    | device has jsut been started. needs ports to be opened
        //vec![0x5, 0x01, 0x01, 0x0, 0x0, 0x0]   | port one selected and idle
        //s1,2 is the port number
        //|bitmap|s1,1|s1,2|s3,1| ,2 |  ,3|
        msg(vec![
            0x5,
            config.port_status.clone() as u8,
            config.number,
            0x0,
            0x0,
            0x0,
        ])
    });
    let select_port: Command = Command::new("select_port", 0x2, 0x22, |message, _, config| {
        info!(
            "Request to select port {:} setting port number to that",
            message.data[0]
        );
        config.number = message.data[0];
        simp(vec![0x04])
    }); //?NOTE this selects a specific port for playing
    let cue_with_data: Command = Command::new("cue_with_data", 0xa, 0x25, |msg, _, config| {
        info!(
            "Cueing clip: {:}",
            std::str::from_utf8(&msg.data[0..6]).unwrap_or("")
        );
        config.port_status = PortStatus::Cued;
        simp(vec![0x04])
    }); //the data is discarded becuase we don't need to cue
    let active_id_request: Command = Command::new("active_id_request", 0x0b, 0x07, active_id); //TODO: i need to find out what this command is for
    let play: Command = Command::new("play", 0x1, 0x01, play); //TODO: my sample from the logs desn't show play as sending a specific port.
    let stop: Command = Command::new("stop", 0x1, 0x00, stop); //TODO: check if we even need to stop
    let id_request: Command = Command::new("id_request", 0xb, 0x16, |message, _, _| {
        match String::from_utf8(message.data.clone()) {
            Ok(a) => info!("Got ID request for file : {:}", a),
            _ => (),
        }
        msg(vec![0x01, 0x00, 0x00]) //i don't know why this must be 3 bytes but it is what we see in the logs
    }); //This just returns 01 to confirm the clip exists
    let commands = vec![
        id_request,
        size_request,
        port_status,
        system_status,
        open_port,
        select_port,
        cue_with_data,
        active_id_request,
        play,
        close_port,
        stop,
    ];
    return commands;
    /*
    Some potential port status states:
    Not connected,
    connected
    cued
    playing
    stoped


    */
}
