//===Adam commmunication module===
use itertools::*;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::*;
use std::sync::mpsc::*;
use std::thread;
use std::{self, io::Error};
use ureq;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdamCommand {
    adam_module: AdamID,
    digital_output_number: u8,
}

impl AdamCommand {
    pub fn new(adam_module: AdamID, digital_output_number: u8) -> Self {
        Self {
            adam_module,
            digital_output_number,
        }
    }
}

type AdamID = u8;
type VDCPPortNum = u8;
type CommandMapping = HashMap<VDCPPortNum, AdamCommand>;
type AdamIPs = HashMap<AdamID, Ipv4Addr>;

fn check_for_config_errors(port_mapping: &CommandMapping, unit_ips: &AdamIPs) {
    for (_, port) in port_mapping {
        if !(unit_ips.contains_key(&port.adam_module)) {
            error!(
                "the adam module {:} doesnt ahve an ip listed in the unit ips given {:?}",
                port.adam_module, unit_ips
            );
        }
    }
}
///Recevies play commands as a u8 representing the port to trigger.
///port_mapping is the adam port associated to each playout port
///unit_ips isthe ip for each adam module that an adam command points to
pub fn start(
    play_commands: Receiver<u8>,
    port_mapping: CommandMapping,
    unit_ips: AdamIPs,
) -> Result<(), Error> {
    info!("Starting adam communicator");
    check_for_config_errors(&port_mapping, &unit_ips);
    info!("adam client setup, starting loop");
    loop {
        thread::sleep(std::time::Duration::from_millis(11));
        //gets all pending values
        let newcomands: Vec<_> = play_commands.try_iter().collect();
        if newcomands.len() > 0 {
            for (address, body) in make_commands(newcomands, &port_mapping, &unit_ips) {
                let form: Vec<(&str, &str)> =
                    body.iter().map(|(a, b)| (a.as_ref(), b.as_ref())).collect();
                send_req(&form, address);
            }
        }
    }
    Ok(())
}

fn send_req(form: &Vec<(&str, &str)>, address: String) {
    //TODO: replace the username and password with something from a config file
    let response = ureq::post(&address).auth("root", "admin").send_form(form);
    match response.ok() {
        false => error!(
            "Request {:}|{:?} to set digital ports on adam failed response: {:?}",
            address, form, response
        ),
        true => info!(
            "Request {:}|{:?} to set digital ports on adam success. Response:{:?}",
            address, form, response
        ),
    }
}

//Takes a play channel id and reutrns the appropriate comamnd to send to the assigned adam
fn make_commands<'a>(
    mut ports_to_play: Vec<u8>,
    mapping: &CommandMapping,
    unit_ips: &AdamIPs,
) -> Vec<(String, Vec<(String, String)>)> {

    ports_to_play.sort_unstable();

    let get_adam_command=
        |port|->Option<_>{
        let comm=mapping.get(port);
        match comm {
            None=>{error!("Port {:?} did not have an associated adam command. Not sending a play request",port);
                 return None;},
            Some(x)=>return Some((x.adam_module, x))
        }  
    };
    let groups = ports_to_play
        .iter()
        .filter_map(get_adam_command)
        .into_group_map();

    let get_adam_ip=|(key,value)|{
        let ip=unit_ips.get(&key);
        match ip {
            None=>{error!("Adam module {:} didn't have an ip adress listed. Not sending play request",key);None},
            Some(x)=>Some((x,value))
        }
    };

    let res: Vec<_> = groups
        .into_iter()
        .filter_map(get_adam_ip)
        .map(|(ip, value)| create_post(*ip, value))
        .collect();
    res
}
fn create_post<'a>(ip: Ipv4Addr, pins: Vec<&AdamCommand>) -> (String, Vec<(String, String)>) {
    let address = format!("http://{0}/digitaloutput/all/value", ip);
    let body: Vec<_> = pins
        .iter()
        .map(|a| (format!("DO{:}", a.digital_output_number), "1".to_string()))
        .collect();
    (address, body)
}


//--------==================================================-----
//=================================TESTS:======================================
//--------==================================================-----

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::{make_commands, AdamCommand, AdamIPs, CommandMapping};

    fn get_test_data() -> (CommandMapping, AdamIPs) {
        let mut mapping = CommandMapping::new();
        let adam_out_1 = AdamCommand::new(0, 0);
        let adam_out_2 = AdamCommand::new(0, 1);
        let adam_out_3 = AdamCommand::new(0, 3);
        let adam_out_4 = AdamCommand::new(1, 0);
        mapping.insert(0, adam_out_1);
        mapping.insert(1, adam_out_2);
        mapping.insert(2, adam_out_3);
        mapping.insert(3, adam_out_4);
        let mut Ips = AdamIPs::new();
        Ips.insert(0, Ipv4Addr::new(10, 0, 0, 1));
        Ips.insert(1, Ipv4Addr::new(10, 0, 0, 2));
        (mapping, Ips)
    }
    #[test]
    fn make_commands_test() {
        let (map, ips) = get_test_data();
        let mut res = make_commands(vec![0, 3], &map, &ips);
         let mut truth:Vec<(&str,Vec<(&str,&str)>)> = vec![
            (
                "http://10.0.0.1/digitaloutput/all/value",
                vec![("DO0", "1")],
            ),
            (
                "http://10.0.0.2/digitaloutput/all/value",
                vec![("DO0", "1")],
            ),
        ];
        let mut conv_truth:Vec<_>=truth.drain(..).map(|(a,mut b)|
        (String::from(a),
            b.drain(..).map(
                |(a,b)|{(String::from(a),String::from(b))}
            ).collect()
        )).collect(); 

       //we sort them both beucase order is not neccissarily preserved
        conv_truth.sort();
        res.sort();
        assert_eq!(res,conv_truth);
    }
}
