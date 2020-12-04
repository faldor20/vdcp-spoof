

use modular_bitfield::prelude::*;
#[bitfield]
#[derive(Clone, Copy)]
pub struct Nibbles {
   pub n1: B4,
   pub n2: B4,
}
#[bitfield]
pub struct Nibble {
   pub data: B4,
    #[skip]
    __: B4,
}

pub union ByteNibbles {
   pub nibbles: Nibbles,
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
   pub name: String,
   pub command_type: Nibble,
   pub command_code: u8,
   pub action: Box<dyn Fn(&Message, &Vec<u16>) -> Vec<u8>>,
}
impl Command {
    pub fn new<T>(name: &str, command_type: u8, command_code: u8, action: T) -> Command
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