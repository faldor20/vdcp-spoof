use std::{
    self,
    error::Error,
    io,
    sync::mpsc::Receiver,
    thread,
    time::{Duration, Instant},
};

use log::*;
use serialport::prelude::*;
use vdcp::types::ClipStatus::NoClips;

use crate::vdcp::{
    self,
    types::{ByteNibbles, Message, PortConfig},
};

pub fn start(
    com: String,
    vdcp_times: Receiver<Vec<u16>>,
    config: PortConfig,
) -> Result<(), Box<dyn Error>> {
    info!("[Port:{0}] Starting serial connection at com port:{1}",config.number, com);
    let port_settings = serialport::SerialPortSettings {
        baud_rate: 38400,
        flow_control: FlowControl::None,
        data_bits: DataBits::Eight,
        parity: Parity::Odd,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };
    let port = serialport::open_with_settings(&com, &port_settings)?;

    serial_reader(port, vdcp_times, config)?;

    Ok(())
}
//an end of a message can be one of two things.
//1: a stx code(02h) denoting teh start of a new message
//3: getting as many bytes as the bytecount +1(the checksum is not counted)
//2: an extended time without new data
fn read_message(port: &mut Box<dyn SerialPort>, byte_count: u8,portNum:u8) -> Result<Message, io::Error> {
    let expected_bytes: usize = (byte_count + 1).into();

    //======Try to read the message and checksum ======
    debug!("[Port:{:}]Reading message of length {:}",portNum, byte_count);
    let mut message_buf = vec![0; expected_bytes];
    let read = port.read(&mut message_buf)?;

    if read != expected_bytes {
        warn!(
            "[Port:{:}]Read command {:?} but it was missing {:?} bytes",portNum,
            message_buf,
            expected_bytes - read
        );
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "didn't read correct number of bytes",
        ));
    //TODO: fire of a NAK respose for an incomplete message.
    //TODO: if this is often off by one it measn there is not allways a checksum.
    } else {
        //=====convert the bytes into a message object=====
        let nibbles = ByteNibbles {
            byte: message_buf[0],
        };
        let mut data = message_buf.split_off(2);
        let checksum = data.split_off(data.len() - 1);
        data.shrink_to_fit();
        let msg = Message {
            byte_count: (byte_count),
            command1: nibbles,
            command_code: message_buf[1],
            data: data,
            checksum: checksum[0],
        };
        //=====Give the message to the vdcp command runner=====
        //TODO: it might be worth starting a new thread here

        Ok(msg)
    }
}
fn handle_message(
    port: &mut Box<dyn SerialPort>,
    msg: Message,
    vdcp_times: &Vec<u16>,
    config: &mut PortConfig,
) -> Result<(), io::Error> {
    let response = vdcp::handle_command(msg, vdcp_times, config);
    debug!("(hex)[Port:{:}] sending response : {:x?}",config.number, response);
    port.write_all(&response)?;
    Ok(())
}
///Reads the byte_count byte from an incoming message
fn read_length(port: &mut Box<dyn SerialPort>,portNum:u8) -> Result<u8, io::Error> {
    debug!("[Port:{:}]Got start of message byte",portNum);
    let mut buf = [0u8; 1];
    port.read_exact(&mut buf)?;
    Ok(buf[0])
}
///Tries to read the start byte of a message, returns error if it doesnt read a start byte
fn read_start(port: &mut Box<dyn SerialPort>,portNum:u8) -> Result<(), io::Error> {
    let mut buf = [0u8; 1];
    port.read_exact(&mut buf)?;
    //check if the byte read is the beginning of a message
    if buf[0] == 0x02 {
    } else {
        warn!(
            "(hex)[Port:{:}]Got byte that wasn't a message start when a start was expected: |{:x?}|",portNum,
            buf[0]
        );
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Byte was not the start of a message",
        ));
    }
    Ok(())
}
///Attempts to read data from the port and then run a command associated with it
fn handle_incoming_data(
    port: &mut Box<dyn SerialPort>,
    vdcp_times: &Vec<u16>,
    config: &mut PortConfig,
) -> Result<(), io::Error> {
    //TODO: make it so taht naything after the readstart causing a faulure sends a NAK back to the sender
    read_start(port,config.number)?; //delay after if fail

    //Tiny delay here just to make sure the data gets to us before we read on.
    //TODO: test if this is necissary
    thread::sleep(std::time::Duration::from_millis(10));

    let byte_count = read_length(port,config.number)?;
    let message = read_message(port, byte_count,config.number)?;
    handle_message(port, message, vdcp_times, config)?;
    Ok(())
}

fn resend_times(config: &mut PortConfig) -> Instant {
    config.clip_status = NoClips;
    info!("[Port:{:}] got new times, setting clips to 0 and waiting 20s",config.number);
    Instant::now()

}
fn check_timeout(
    timeout: &mut Option<Instant>,
    timeout_length: &Duration,
    config: &mut PortConfig,
) {
    match timeout {
        Some(x) => {
            
            if Instant::now().duration_since(*x) > *timeout_length {
                info!("[Port:{:}] Timeout elapsed setting clips back to 1f",config.number);
                *timeout = None;
                config.clip_status = vdcp::types::ClipStatus::Clips
            }
        }
        _ => (),
    }
}

fn serial_reader(
    mut port: Box<dyn SerialPort>,
    vdcp_times: Receiver<Vec<u16>>,
    mut config: PortConfig,
) -> Result<(), std::io::Error> {
    info!("[Port:{:}] About to start read loop",config.number);
    //currently this just keeps reading till it finds a beginning of message command
    let mut latest_times: Vec<u16> = vec![0; 10]; //todo: setting this with a random number could result in trying to access a time out of range
    let mut timeout: Option<Instant> = Option::None;
    let timeout_length = Duration::from_secs(20);
    loop {
        check_timeout(&mut timeout, &timeout_length, &mut config);

        //we have to unwrap the thread safe atomic cell and read
        let times = vdcp_times.try_iter();
        match times.last() {
            Some(x) => {
                let port_name = &*port.name().unwrap_or_default();
                info!("[Port:{:}] Got new times data {:?} for port {:}",config.number, &x, port_name);
                latest_times = x;
                timeout=Some(resend_times(&mut config));
            }
            _ => (),
        }
        match handle_incoming_data(&mut port, &latest_times, &mut config) {
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut => continue,
                _ => warn!("[Port:{:}] message read failed becuase: {:}",config.number, e),
            },
            Ok(_) => (),
        }
        thread::sleep(std::time::Duration::from_millis(5));
    }
}
