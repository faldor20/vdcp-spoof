use modular_bitfield::prelude::*;
use log::*;
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
    name:String,
    command_type: Nibble,
    command_code: u8,
    action: Box<dyn Fn(Message) -> Vec<u8>>,
}
impl Command {
    fn new<T>(name: &str,command_type: u8, command_code: u8, action: T) -> Command
    where
        T: Fn(Message) -> Vec<u8> + 'static,
    {
        //let a:|Message|->()=|x:Message|{return;};

        Command {
            name:name.into(),
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
fn play(message: Message)->Vec<u8> {
info!("Playing port");
vec![0x04]
}
fn active_id(message: Message)->Vec<u8> {
    vec![0x04]
}
fn stop(message: Message)->Vec<u8> {
    vec![0x04]
}
fn size_request(message:Message)->Vec<u8> {
    //TODO: get a length for the clip name asked about
    vec![0xb0, 0x94, 0x07, 0x00, 0x00]
}
fn unknown_command(msg: Message)->Vec<u8> {
    unsafe{
    warn!("(hex)received unknown command|{:x?}|{:x?}|{:x?}|{:x?}|{:x?}|",msg.byte_count,msg.command1.byte,msg.command_code,msg.data,msg.checksum);
    }
    vec![0x05,0x0]
}
fn run_command(message: Message, commands: &[Command]) ->Vec<u8>{
    for command in commands {
        //we have to use an unsafe block becuase we access a union to get our nibbles from a byte
        unsafe {
           

            if message.command1.nibbles.n2() == command.command_type.data()
                && message.command_code == command.command_code
            {
                info!("running command {:}",command.name);
                let func = &*command.action;

                let a=func(message);
                return a;
            }
        }
    }
    unknown_command(message)
}




pub fn Run_Command(msg: Message)->Vec<u8> {
    
    let size_request:Command      = Command::new("size_request",0xb,0x14,size_request);
    let system_status:Command     = Command::new("system_status",0x3,0x10,|_|(vec![0x30, 0x90, 0x02 ,0x00, 0x1f]));  //?NOTE: The return here is the number of ids stored by the vdcp server. i think it can remain constant and simply be the max number of clips we ever have
    let open_port:Command         = Command::new("open_port",0x3,0x14,|_|( vec![0x30, 0x81, 0x01,]));
    let something:Command         = Command::new("something",0x2,0x22,|_|(vec![0x04]));                              //TODO: find out what this does
    let cue_with_data:Command     = Command::new("cue_with_data",0xa,0x25,|_|(vec![0x04]));                          //the data is discarded becuase we don't need to cue
    let active_id_request:Command = Command::new("active_id_request",0x0b,0x07,active_id);                           //TODO: i need to find out what this command is for
    let play:Command              = Command::new("play",0x1,0x01,play);                                              //TODO: my sample from the logs desn't show play as sending a specific port.
    let stop:Command              = Command::new("stop",0x1,0x00,stop);                                              //TODO: check if we even need to stop
    let id_request:Command        = Command::new("id_request",0xb,0x16,|_|(vec![0xb0, 0x96, 0x01, 0x00, 00]));       //This just returns 01 to confirm the clip exists
    
    unsafe{
        info!("(hex)Processing command for message:|{:x?}|{:x?}[{:x?}/{:x?}]|{:x?}|{:x?}|{:x?}|",msg.byte_count,msg.command1.byte,msg.command1.nibbles.n2(),msg.command1.nibbles.n1(),msg.command_code,msg.data,msg.checksum);    }

    let commands = [
      id_request,
      size_request,
      system_status,
      open_port,
      something,
      cue_with_data,
      active_id_request,
      play,
      stop

        ];
    run_command(msg, &commands)
}
