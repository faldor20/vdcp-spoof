//===Adam commmunication module===
use std::{self, io::Error};
use std::net::*;
use std::collections::HashMap;
use ureq;
use std::sync::mpsc::*;
use std::thread;
use itertools::*;
use log::*;
use serde::{Deserialize, Serialize};
#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct AdamCommand{
    adam_module:u8,
    digital_output_number:u8,


}
fn check_for_config_errors(port_mapping:&HashMap<u8,AdamCommand>,unit_ips:&HashMap<u8,Ipv4Addr> ){
    for (_,port) in port_mapping{
        if !(unit_ips.contains_key( &port.adam_module)){
            error!("the adam module {:} doesnt ahve an ip listed in the unit ips given {:?}",port.adam_module,unit_ips);
        }
    }

}
///Receies play commands as a u8 representing the port to trigger.
///port_mapping is the adam port associated to each playout port
///unit_ips isthe ip for each adam module that an adam command points to
pub fn start(play_commands:Receiver<(u8)>, port_mapping:HashMap<u8,AdamCommand>,unit_ips:HashMap<u8,Ipv4Addr> )->Result<(),Error>{
    info!("Starting adam communicator");
    check_for_config_errors(&port_mapping, &unit_ips);
    info!("adam client setup, starting loop");
    loop{
        thread::sleep(std::time::Duration::from_millis(11));
        //gets all pending values
        let newcomands:Vec<_>=play_commands.try_iter().collect();
        if newcomands.len()>0 {
           for (address,body) in make_commands(newcomands,&port_mapping,&unit_ips){
               //TODO: replace the username and password with something from a config file
                let form: Vec<(&str,&str)> = body.iter().map(|(a,b)|(a.as_ref(),b.as_ref())).collect();
                let res=ureq::post(&address).auth("root", "admin").send_form(&form);
                match res.ok() {
                    false=>error!("Request {:}|{:?} to set digital ports on adam failed response: {:?}",address,body, res),
                   true=>info!("Request {:}|{:?} to set digital ports on adam success. Response:{:?}",address,body,res),
                }
           }
        }
    }
 Ok(())
}
//Takes a play channel id and reutrns the appropriate comamnd to send to the assigned adam
fn make_commands<'a>(mut nums:Vec<u8>,mapping:&HashMap<u8,AdamCommand>,unit_ips:&HashMap<u8,Ipv4Addr>)->Vec<(String,Vec<(String,String)>)>{
    nums.sort_unstable();
    let groups=nums.iter().map(|port|(mapping[port].adam_module,&mapping[port])).into_group_map();
    let res:Vec<_>=groups.into_iter().map(|(key,value)|{create_post(unit_ips[&key],value)}).collect();
    res
}
fn create_post<'a>(ip:Ipv4Addr,pins:Vec<&AdamCommand>)->(String,Vec<(String,String)>){
let address=format!("http://{0}/digitaloutput/all/value",ip);
let body:Vec<_>= pins.iter().map(|a|(format!("DO{:}",a.digital_output_number),"1".to_string())).collect();
(address,body)
}
//we will use ascII to coomunicate. commands we care about are:
//#aaDnd| Set single digital output status |Sets the output status of a specific digital output channel
//the comamnd format is as follows
//#01Dnd | the 01 is the module it targets. This is allwasy 01
//#01D


