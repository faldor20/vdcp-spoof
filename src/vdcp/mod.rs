
use log::*;
use types::*;

use responses::unknown_command;
mod responses;
pub mod types;
/*
*Implimentation notes:use std::{error::Error, str::from_utf8};

use log::*;
use modular_bitfield::prelude::*;
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


fn checksum(bytes: &Vec<u8>) -> u8 {
    let sum: u32 = bytes.iter().fold(0u32, |x, y| {
        let a: u32 = y.clone().into();
        a + x
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

pub fn handle_command(msg: Message, clip_times: &Vec<u16>) -> Vec<u8> {

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

    let commands = responses::get_commands();
    let return_data = run_command(&msg, &commands, clip_times);
    let return_message = post_processing(&msg, return_data);
    return return_message;
}
