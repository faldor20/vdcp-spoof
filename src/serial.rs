
use std::{self, error::Error, io, time::Duration, thread};

use crossbeam::atomic::AtomicCell;
use log::*;
use serialport::prelude::*;

use crate::vdcp::{self, Message};
use super::TimesDB;
pub fn start(com: &String,vdcp_times:&AtomicCell<Vec<u16>>) -> Result<(), Box<dyn Error>> {
    info!("starting serial connection at com port:{0}", com);
    let port_settings = serialport::SerialPortSettings {
        baud_rate: 38400,
        flow_control: FlowControl::None,
        data_bits: DataBits::Eight,
        parity: Parity::Odd,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };
    let port = serialport::open_with_settings(&com, &port_settings)?;
    serial_reader(port,vdcp_times)?;
    Ok(())
}
//an end of a message can be one of two things.
//1: a stx code(02h) denoting teh start of a new message
//3: getting as many bytes as the bytecount +1(the checksum is not counted)
//2: an extended time without new data
fn read_message(port: &mut Box<dyn SerialPort>, byte_count: u8) -> Result<Message, io::Error> {
    let expected_bytes: usize = (byte_count + 1).into();

    //======Try to read the message and checksum ======
    debug!("Reading message of length {0}", byte_count);
    let mut message_buf = vec![0; expected_bytes];
    let read = port.read(&mut message_buf)?;

    if read != expected_bytes {
        warn!(
            "Read command {:?} but it was missing {:?} bytes",
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
        let nibbles = vdcp::ByteNibbles {
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
fn handle_message(port: &mut Box<dyn SerialPort>,msg:Message,vdcp_times:&Vec<u16>)->Result<(),io::Error>{
    let response = vdcp::Run_Command(msg,vdcp_times);
    port.write_all(&response)?;
    Ok(())
}
///Reads the byte_count byte from an incoming message
fn read_length(port: &mut Box<dyn SerialPort>) -> Result<u8, io::Error> {
    debug!("Got start of message byte");
    let mut buf = [0u8; 1];
    port.read_exact(&mut buf)?;
    Ok(buf[0])
}
///Tries to read the start byte of a message, returns error if it doesnt read a start byte
fn read_start(port: &mut Box<dyn SerialPort>) -> Result<(), io::Error> {
    let mut buf = [0u8; 1];
    port.read_exact(&mut buf)?;
    //check if the byte read is the beginning of a message
    if buf[0] == 0x02 {
    } else {
        warn!(
            "(hex)Got byte that wasn't a message start when a start was expected{:x?}",
            buf[0]
        );
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "byte was not the start of a message",
        ));
    }
    Ok(())
}
///Attempts to read data from the port and then run a command associated with it
fn handle_incoming_data(port: &mut Box<dyn SerialPort>,vdcp_times:&Vec<u16>) -> Result<(), io::Error> {
    //TODO: make it so taht naything after the readstart causing a faulure sends a NAK back to the sender
    read_start(port)?; //delay after if fail

    //Tiny delay here just to make sure the data gets to us before we read on.
    //TODO: test if this is necissary
    thread::sleep(std::time::Duration::from_millis(10));

    let byte_count = read_length(port)?;
    let message= read_message(port, byte_count)?;
    handle_message(port,message,vdcp_times)?;
    Ok(())
}
fn serial_reader(mut port: Box<dyn SerialPort>,vdcp_times:&AtomicCell<Vec<u16>>) -> Result<(), std::io::Error> {
    //currently this just keeps reading till it finds a beginning of message command
    
    loop {
        //todo: this code is ugly as. but i can't use load to get the var becuase vec isn't able to bopied, only cloned
        //we have to unwrap the thread safe atomic cell and read
        let times= vdcp_times.take(); 
        vdcp_times.store(times.clone());

        match handle_incoming_data(&mut port,&times) {
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut => continue,
                _ => warn!("message read failed becuase:{0}", e),
            },
            Ok(_) => (),
        }
        thread::sleep(std::time::Duration::from_millis(5));
    }
}
fn old_loop(mut port: Box<dyn SerialPort>,vdcp_times:&Vec<u16>) -> Result<(), Box<dyn Error>> {
    //=====Try to read the beginning of message byte=======
    let mut buf = [0u8; 1];
    let read = port.read(&mut buf)?;
    //make sure we got something
    if read > 0 {
        //Tiny delay here just to make sure the data gets to us before we read on.
        //TODO: test if this is necissary
        thread::sleep(std::time::Duration::from_millis(10));
        //check if the byte read is the beginning of a message
        if buf[0] == 0x02 {
            debug!("Got start of message byte");
            //=====Try to read the mesage length byte=====
            let mut buf = [0u8; 1];
            let read = port.read(&mut buf)?;
            //Make sure we got something
            if read > 0 {
                let bytecount = buf[0];
                let expected_bytes: usize = (bytecount + 1).into();

                //======Try to read the message and checksum ======
                debug!("Reading message of length {0}", bytecount);
                let mut message_buf = vec![0; expected_bytes];
                let read = port.read(&mut message_buf)?;

                if read != expected_bytes {
                    warn!(
                        "Read command (hex){:x?} but it was missing {:?} bytes",
                        message_buf,
                        expected_bytes - read
                    )
                //TODO: fire of a NAK respose for an incomplete message.
                //TODO: if this is often off by one it measn there is not allways a checksum.
                } else {
                    //=====convert the bytes into a message object=====
                    let nibbles = vdcp::ByteNibbles {
                        byte: message_buf[0],
                    };
                    let mut data = message_buf.split_off(2);
                    let checksum = data.split_off(data.len() - 1);
                    data.shrink_to_fit();
                    let msg = Message {
                        byte_count: (bytecount),
                        command1: nibbles,
                        command_code: message_buf[1],
                        data: data,
                        checksum: checksum[0],
                    };
                    //=====Give the message to the vdcp command runner=====
                    //TODO: it might be worth starting a new thread here

                    let response = vdcp::Run_Command(msg,vdcp_times);
                    port.write_all(&response)?;
                }
            } else {
                error!("got start of message and then no more bytes")
            }
        } else {
            warn!(
                "(hex)Got byte that wasn't a message start when a start was expected{:x?}",
                buf[0]
            );
        }
    } else {
        //do a delay here to stop continously reading
    }
    Ok(())
}
