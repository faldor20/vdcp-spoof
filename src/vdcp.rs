use std::{error::Error, str::from_utf8};

use log::*;
use modular_bitfield::prelude::*;
#[bitfield]
#[derive(Clone, Copy)]
struct Nibbles {
    n1: B4,
    n2: B4,
}
#[bitfield]
struct Nibble {
    data: B4,
    #[skip]
    __: B4,
}

pub union ByteNibbles {
    nibbles: Nibbles,
    pub byte: u8,
}

pub struct Message {
    pub byte_count: u8,
    pub command1: ByteNibbles,
    pub command_code: u8,
    pub checksum: u8,
    pub data: Vec<u8>,
}

pub struct Command {
    name: String,
    command_type: Nibble,
    command_code: u8,
    action: Box<dyn Fn(&Message, &Vec<u16>) -> Vec<u8>>,
}
impl Command {
    fn new<T>(name: &str, command_type: u8, command_code: u8, action: T) -> Command
    where
        T: Fn(&Message, &Vec<u16>) -> Vec<u8> + 'static,
    {
        //let a:|Message|->()=|x:Message|{return;};

        Command {
            name: name.into(),
            command_type: Nibble::new().with_data(command_type),
            command_code: command_code,
            action: Box::new(action),
        }
    }
}
/*
*Implimentation notes:
*all commands must return an ACK after receiving
*non comprehnded commands should return NAK
*
*Message format:
* v stx (02h)    v cmd type  v command code
*|8     |8     |[4   ][4   ]|8    |
*        ^count        ^ unit address
*
*Byte  Count  (BC)[8]:  Indicates  the  number  of  bytes  between  the  byte  count  and  thechecksum
*Command Type [4]:
*Unit Adress[4]:Defines the address of a sub-system within the device. The base unit is, 0.
*command code[8]: Defines what command should be run
*/
/*
*Aknowledgements:
all commands not requireing a message response must send back an ACK
The response to command types 0, 1, and 2 will be an ACK (04h) or NAK (05h).
The responseto command type 3 will set the most significant bit of the command to a 1,
e.g. the response tocommand 29 is A9. The command codes form a unique device dialect.

*/

/*
2.Bit Configuration
a.1 start bit (space)
b.8 data bits
c.1 parity bit (odd)
d.1 stop bit (mark)
e.Byte time = .286 msec.
*/
fn play(message: &Message, _: &Vec<u16>) -> Vec<u8> {
    info!("Playing port");
    vec![0x04]
}
fn active_id(message: &Message, _: &Vec<u16>) -> Vec<u8> {
    vec![0x04]
}
fn stop(message: &Message, _: &Vec<u16>) -> Vec<u8> {
    vec![0x04]
}
fn size_request(message: &Message, clip_times: &Vec<u16>) -> Vec<u8> {
    let clip_name = from_utf8(&message.data).unwrap_or("failed to convert from bytes to utf8");
    info!("size requested for clip {:?}", clip_name);
    let stuff = || -> Result<Vec<u8>, Box<dyn Error>> {
        //the last data byte should tell us the clip number as a utf8 byte
        let last = message.data.last().ok_or("data was empty")?;

        //if we take a utf8 number char byte and subtract hex 30 from it it becomes the number the charctor is
        let index: usize = (last - 0x30) as usize;

        let a = clip_times
            .get(index)
            .ok_or("clip time requested didn't exist")?;
        //This gets our hours and then seconds.
        let minutes = a / 60u16;
        let seconds = a - (minutes * 60u16);
        info!("clip {:} is {:}:{:}", clip_name, minutes, seconds);
        //data is: frames|seconds|minutes|hours
        Ok(vec![0x00, seconds as u8, minutes as u8, 0x00])
    };

    stuff().unwrap_or_else(|err: Box<dyn Error>| {
        error!("Failed processing size request. Reason: {:?}", err);
        vec![0x05]
    })
}
fn unknown_command(msg: &Message) -> Vec<u8> {
    unsafe {
        warn!(
            "(hex)received unknown command|{:x?}|{:x?}|{:x?}|{:x?}|{:x?}|",
            msg.byte_count, msg.command1.byte, msg.command_code, msg.data, msg.checksum
        );
    }
    vec![0x05,0x0]
}

fn checksum(bytes: &Vec<u8>) -> u8 {
    let sum: u32 = bytes.iter().fold(0u32, |x, y| {
        let a: u32 = y.clone().into();
        (a + x)
    });
  
    //we need only the least significan't byte
    let x: u8 = sum.to_le_bytes()[0];
   
    //this gives us the twos compliment. This method is confirmed by the messages recieved form teh actuall vdcp server.
    //it flips all the bits and adds one
    let compliment = x.wrapping_neg();
    return compliment;
}

///surrounds return data with the approprate stuff to mkae it a valid message
///this does:
///1. 0x02 starts the message. to indicate a message start.
///2. Counts the data bytes and adds 2 to to take into account the 2 command bytes
///   and appeneds it as the "byte count"
///3. Appends the original command type
///4. Appeneds the comamnd code:Command codes other than 0,1and 2 wwill require the mos significant bit being set to one
///   This is eqivilent to adding 128 or 0x80
///5.Appends the data
/// calculates a checksum from the command and data bytes by:
///     a. Summing the command and data bytes
///     b. Taking the least significant byte of the result
///     c. get the 2's compliment of that.
fn post_processing(message: &Message, mut data: Vec<u8>) -> Vec<u8> {
    // let base=-vec![0x02,command,commad2];
    // Vec::append(data)
        unsafe {
            match message.command1.byte {
        0x0 | 0x10 | 0x20 => {
            //TODO: make sure this is correct
            return vec![ data[0]];
        }
        _ => {
                let command_1=message.command1.byte;
                //we + 0x80 to set the most significant bit to 1
                let mut body: Vec<u8> = vec![command_1 , message.command_code + 0x80];
                body.append(&mut data);
                let bc = body.len().to_le_bytes()[0];
                
                //caculate checksum
                let checksum = checksum(&body);
                body.push(checksum);
                
                let mut start: Vec<u8> = vec![0x02, bc];
                start.append(&mut body);
                
                return start;
            }
        }
    }
}

fn run_command(message: &Message, commands: &[Command], clip_times: &Vec<u16>) -> Vec<u8> {
    for command in commands {
        //we have to use an unsafe block becuase we access a union to get our nibbles from a byte
        unsafe {
            if message.command1.nibbles.n2() == command.command_type.data()
                && message.command_code == command.command_code
            {
                info!("Running command: '{:}'", command.name.to_uppercase());
                let func = &*command.action;

                let a = func(&message, clip_times);
                return a;
            }
        }
    }
    unknown_command(message)
}

pub fn Run_Command(msg: Message, clip_times: &Vec<u16>) -> Vec<u8> {
    let size_request: Command = Command::new("size_request", 0xb, 0x14, size_request);
    let system_status: Command =
        Command::new("system_status", 0x3, 0x10, |_, _| (vec![0x02, 0x00, 0x1f])); //?NOTE: The return here is the number of ids stored by the vdcp server. i think it can remain constant and simply be the max number of clips we ever have
    let open_port: Command = Command::new("open_port", 0x3, 0x01, |_, _| (vec![0x01]));

    let port_status: Command = Command::new("port_status", 0x3, 0x05, |_, _| {
        //
        (vec![0x5, 0x0, 0x0, 0x0, 0x01, 0x01])
        
    });
    let select_port: Command = Command::new("select_port", 0x2, 0x22, |_, _| (vec![0x04])); //?NOTE this selects a specific port for playing
    let cue_with_data: Command = Command::new("cue_with_data", 0xa, 0x25, |_, _| (vec![0x04])); //the data is discarded becuase we don't need to cue
    let active_id_request: Command = Command::new("active_id_request", 0x0b, 0x07, active_id); //TODO: i need to find out what this command is for
    let play: Command = Command::new("play", 0x1, 0x01, play); //TODO: my sample from the logs desn't show play as sending a specific port.
    let stop: Command = Command::new("stop", 0x1, 0x00, stop); //TODO: check if we even need to stop
    let id_request: Command = Command::new("id_request", 0xb, 0x16, |_, _| {
        (vec![0xb0, 0x96, 0x01, 0x00, 00])
    }); //This just returns 01 to confirm the clip exists

    unsafe {
        info!(
            "(hex)Processing command for message:|{:x?}|{:x?}[{:x?}/{:x?}]|{:x?}|{:x?}|{:x?}|",
            msg.byte_count,
            msg.command1.byte,
            msg.command1.nibbles.n2(),
            msg.command1.nibbles.n1(),
            msg.command_code,
            msg.data,
            msg.checksum
        );
    }

    let commands = [
        id_request,
        size_request,
        port_status,
        system_status,
        open_port,
        select_port,
        cue_with_data,
        active_id_request,
        play,
        stop,
    ];
    let return_data = run_command(&msg, &commands, clip_times);
    let return_message = post_processing(&msg, return_data);
    return return_message;
}
