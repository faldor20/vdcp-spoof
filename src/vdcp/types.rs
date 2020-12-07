

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

#[derive(Clone)]
#[repr(u8)]
pub enum PortStatus{
    Idle=0x01,
    Cued=0x80,
    Playing=0x04,

}
pub struct PortConfig{
 pub number:u8,
 pub port_status:PortStatus,
}
pub enum Response{
    Message(Vec<u8>),
    Simple(Vec<u8>)

    
}
impl Into<Vec<u8>> for Response{
  fn into(self)->Vec<u8>{  match self {
        Response::Message(a)=>a,
        Response::Simple(a)=>a
    }
}
}
pub struct Command {
   pub name: String,
   pub command_type: Nibble,
   pub command_code: u8,
   pub action: Box<dyn Fn(&Message, &Vec<u16>,&mut PortConfig) ->Response >,
}
impl Command {
    pub fn new<T>(name: &str, command_type: u8, command_code: u8, action: T) -> Command
    where
        T: Fn(&Message, &Vec<u16>,&mut PortConfig) -> Response + 'static,
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