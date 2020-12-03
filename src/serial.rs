use std::{
    self,
    error::{self, Error},
    fmt, io,
    sync::mpsc::Receiver,
    thread,
    time::Duration,
};

use crossbeam::atomic::AtomicCell;
use log::*;
use serialport::prelude::*;

use crate::vdcp::{self, Message};

pub fn start(com: String, vdcp_times: Receiver<Vec<u16>>) -> Result<(), Box<dyn Error>> {
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
    serial_reader(port, vdcp_times)?;
    Ok(())
}
#[derive(Debug)]
enum SerialError {
    Io(std::io::Error),
    UnexpectedStart,
}
impl fmt::Display for SerialError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerialError::Io(e) => write!(f, "IO Error : {:?}", e),
            // The wrapped error contains additional information and is available
            // via the source() method.
            SerialError::UnexpectedStart => {
                write!(f, "Found a messagestart byte where no start was expected")
            }
        }
    }
}
impl error::Error for SerialError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            SerialError::UnexpectedStart => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            SerialError::Io(ref e) => Some(e),
        }
    }
}
impl From<io::Error> for SerialError {
    fn from(err: io::Error) -> SerialError {
        SerialError::Io(err)
    }
}

fn read_serial<'a>(port: &mut Box<dyn SerialPort>, buf: &'a mut [u8]) -> Result<(), SerialError> {
    let mut buf2 = [0u8; 0];
    for (i) in 0..buf.len() - 1 {
        port.read_exact(&mut buf2)?;
        let byte = buf2[0];
        if byte == 0x02 {
            return Err(SerialError::UnexpectedStart);
        }
        buf[i] = byte;
    }
    return Ok(());
}
//an end of a message can be one of two things.
//1: a stx code(02h) denoting teh start of a new message
//3: getting as many bytes as the bytecount +1(the checksum is not counted)
//2: an extended time without new data
fn read_message(port: &mut Box<dyn SerialPort>, byte_count: u8) -> Result<Message, SerialError> {
    let expected_bytes: usize = (byte_count + 1).into();

    //======Try to read the message and checksum ======
    debug!("Reading message of length {0}", byte_count);
    let mut message_buf = vec![0; expected_bytes];
    let read = read_serial(port, &mut message_buf);

    match read {
        Err(e) => {
            match e {
                SerialError::Io(e) if e.kind() == io::ErrorKind::TimedOut => {
                    warn!(
                        "Read command {:?} but couldn't read as many as was required",
                        message_buf,
                    );
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "didn't read correct number of bytes",
                    )
                    .into());
                    //TODO: fire of a NAK respose for an incomplete message.
                    //TODO: if this is often off by one it measn there is not allways a checksum.
                }
                e => Err(e),
            }
        }
        Ok(_) => {
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
}

fn handle_message(
    port: &mut Box<dyn SerialPort>,
    msg: Message,
    vdcp_times: &Vec<u16>,
) -> Result<(), io::Error> {
    let response = vdcp::Run_Command(msg, vdcp_times);
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
fn handle_incoming_data(port: &mut Box<dyn SerialPort>, vdcp_times: &Vec<u16>) -> () {
    let mut thing = || -> Result<(), SerialError> {
        //TODO: make it so taht naything after the readstart causing a faulure sends a NAK back to the sender

        //Tiny delay here just to make sure the data gets to us before we read on.
        //TODO: test if this is necissary
        thread::sleep(std::time::Duration::from_millis(10));

        let byte_count = read_length(port)?;
        let message = read_message(port, byte_count)?;
        handle_message(port, message, vdcp_times)?;
        Ok(())
    };
    match thing() {
        Err(e) => match e {
            //We start the same function again because we already have a message start and don't need to search for another one
            UnexpectedStart => handle_incoming_data(port, vdcp_times),
            e => warn!("message read failed becuase:{0}", e),
        },
        Ok(_) => (),
    }
}
fn serial_reader(
    mut port: Box<dyn SerialPort>,
    vdcp_times: Receiver<Vec<u16>>,
) -> Result<(), std::io::Error> {
    //currently this just keeps reading till it finds a beginning of message command
    let mut latest_times: Vec<u16> = vec![0; 10]; //todo: setting this with a random number could result in trying to access a time out of range

    loop {
        //we have to unwrap the thread safe atomic cell and read
        let times = vdcp_times.try_iter();
        match times.last() {
            Some(x) => {
                let port_name = &*port.name().unwrap_or_default();
                info!("Got new times data {:?} for port {:}", &x, port_name);
                latest_times = x;
            }
            _ => (),
        }

        let start = read_start(&mut port);
        match start {
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => error!("Got error: Whilst readingserial for a message start {:}", e),
            _ => (),
        }
        handle_incoming_data(&mut port, &latest_times);

        thread::sleep(std::time::Duration::from_millis(5));
    }
}
