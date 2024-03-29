use crossbeam::queue::ArrayQueue;
//===Adam communication module===
use itertools::*;
#[cfg(not(test))]
use log::{error, info, warn};

#[cfg(test)]
use std::{println as info, println as warn, println as error};

use rayon::prelude::*;

use serde::{Deserialize, Serialize};
use std::sync::{mpsc::*, Arc};
use std::thread;
use std::{self, io::Error};
use std::{collections::HashMap };
use std::{net::*, time::Duration};
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
type URL = String;
type Key = String;
type Value = String;
type FormData = Vec<(Key, Value)>;
#[derive(Debug)]
enum RequestType {
    Pulse,
}
///Logs errors if the config is in some way broken.
fn check_for_config_errors(port_mapping: &CommandMapping, unit_ips: &AdamIPs) {
    for (_, port) in port_mapping {
        if !(unit_ips.contains_key(&port.adam_module)) {
            error!(
                "the adam module {:} doesn't have an ip listed in the unit ips given {:?}",
                port.adam_module, unit_ips
            );
        }
    }
}

///Will wait for info to come in on the `play_commands` channel and trigger the appropriate port in response.
///
///`play_commands` A channel that receives play commands as a u8 representing the port to trigger play on.
///
///`port_mapping` is the adam port associated to each playout port
///
///`unit_ips` is the ip for each adam module that an adam command points to
///
/// 
pub fn start(
    play_commands: Receiver<u8>,
    port_mapping: CommandMapping,
    unit_ips: AdamIPs,
) -> Result<(), RecvError> {
    info!("Starting adam communicator");
    check_for_config_errors(&port_mapping, &unit_ips);
    info!("adam client setup, starting loop");
    //continuous loop where incoming adam trigger requests sent by the vdcp apart of the program are handled/
    //let mut time=std::time::Instant::now();
    //let buffer=ArrayQueue::new(port_mapping.len()+1);
    let thread_pool=rayon::ThreadPoolBuilder::new().num_threads(5).build().expect("Adam Thread pool failed to be created");


    loop{
         //We wait until we receive a command and then wait for any others that should be executed at the same time
        let first=play_commands.recv()?;
        thread::sleep(std::time::Duration::from_millis(11));
        let mut rest:Vec<_>=play_commands.try_iter().collect();
        rest.append(&mut vec![first]);

        let adam_requests = make_commands(rest, &port_mapping, &unit_ips);
       thread_pool.spawn( move ||{dispatch_adam_requests(adam_requests)})
    }
    
}

fn dispatch_adam_requests(commands: Vec<(RequestType, URL, FormData)>) {
    commands.into_par_iter().for_each(|(req_type,address, body)|{
        let  form: Vec<(&str, &str)> = body.iter().map(|(a, b)| (a.as_ref(), b.as_ref())).collect();
        info!("{{Adam}} Sending Request {:} | {:?}",&address,&form);
        send_req(&form,&address);
        //If it was a pulse we wait a little while then switch the port back to its original state
        match req_type{RequestType::Pulse =>{
            thread::sleep(Duration::from_millis(20)); 
            let off_form:Vec<(&str, &str)>= form.iter().map(|(a,b)|{
                    match *b {
                        "1"=> (*a,"0"),
                        "0"=> (*a,"1"),
                        _=>{
                          error!("Could not run pulse. Sending data that cannot be reversed, simply resending message"); 
                            (*a,*b)
                        }
                    }
                }).collect();
            send_req(&off_form, &address);},
        }
    }
    );
}
///Just a wrapper around ureq takes a http form and sends it.
///see the `send_form` documentation in ureq for details
fn send_req(form: &Vec<(&str, &str)>, address: &URL) {
    //TODO: replace the username and password with something from a config file
    let response = ureq::post(&address).auth("root", "admin").send_form(form);
    match response.ok() {
        false => error!(
            "{{Adam}}Request {:} | {:?} to set digital ports on adam failed response: {:?}",
            address, form, response
        ),
        true => info!(
            "{{Adam}}Request {:} | {:?} to set digital ports on adam success. Response:{:?}",
            address, form, response
        ),
    }
}

///Takes a play channel id and returns the appropriate command to send to the assigned adam
fn make_commands<'a>(
    mut ports_to_play: Vec<u8>,
    mapping: &CommandMapping,
    unit_ips: &AdamIPs,
) -> Vec<(RequestType, URL, FormData)> {
    ports_to_play.sort_unstable();

    let get_adam_command = |port| -> Option<_> {
        let command = mapping.get(port);
        match command {
            None => {
                error!(
                    "Port {:?} did not have an associated adam command. Not sending a play request",
                    port
                );
                return None;
            }
            Some(this_command) => {
                info!(
                    "{{Adam}}Creating play command for port {:} with adam:{:?} ",
                    port, this_command
                );
                return Some((this_command.adam_module, this_command));
            }
        }
    };
    let groups = ports_to_play
        .iter()
        .filter_map(get_adam_command)
        .into_group_map();

    let get_adam_ip = |(key, commands)| {
        let ip = unit_ips.get(&key);
        match ip {
            None => {
                error!(
                    "Adam module {:} didn't have an ip address listed. Not sending play request",
                    key
                );
                None
            }
            Some(x) => Some((x, commands)),
        }
    };

    let res: Vec<_> = groups
        .into_iter()
        .filter_map(get_adam_ip)
        .map(|(ip, value)| create_post(*ip, value))
        .collect();
    res
}
///Creates a http request string for the adam ip and pins given
///It combines all the commands together int a single request
fn create_post<'a>(ip: Ipv4Addr, pins: Vec<&AdamCommand>) -> (RequestType, URL, FormData) {
    let address: URL = format!("http://{0}/digitaloutput/all/value", ip);
    let body: Vec<_> = pins
        .iter()
        .map(|a| (format!("DO{:}", a.digital_output_number), "1".to_string()))
        .collect();
    (RequestType::Pulse, address, body)
}

//--------==================================================-----
//=================================TESTS:======================================
//--------==================================================-----

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use test_env_log::test;

    use super::*;

    fn get_test_data(ip1: Ipv4Addr, ip2: Ipv4Addr) -> (CommandMapping, AdamIPs) {
        let mut mapping = CommandMapping::new();
        let adam_out_1 = AdamCommand::new(0, 0);
        let adam_out_2 = AdamCommand::new(0, 1);
        let adam_out_3 = AdamCommand::new(0, 3);
        let adam_out_4 = AdamCommand::new(1, 0);
        mapping.insert(0, adam_out_1);
        mapping.insert(1, adam_out_2);
        mapping.insert(2, adam_out_3);
        mapping.insert(3, adam_out_4);
        let mut ips = AdamIPs::new();
        ips.insert(0, ip1);
        ips.insert(1, ip2);
        (mapping, ips)
    }
    fn get_commands() -> Vec<(RequestType, URL, FormData)> {
        let (map, ips) = get_test_data(Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 2));
        let res = make_commands(vec![0, 3], &map, &ips);
        return res;
    }
    /*     #[test]
    fn make_commands_test() {
        let mut res = get_commands();
        let mut truth: Vec<(&str, Vec<(&str, &str)>)> = vec![
            (
                "http://10.0.0.1/digitaloutput/all/value",
                vec![("DO0", "1")],
            ),
            (
                "http://10.0.0.2/digitaloutput/all/value",
                vec![("DO0", "1")],
            ),
        ];
        let mut conv_truth: Vec<_> = truth
            .drain(..)
            .map(|(a, mut b)| {
                (
                    String::from(a),
                    b.drain(..)
                        .map(|(a, b)| (String::from(a), String::from(b)))
                        .collect(),
                )
            })
            .collect();

        //we sort them both because order is not necessarily preserved
        conv_truth.sort();
        res.sort();
        assert_eq!(res, conv_truth);
    } */
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
    #[test]
    fn trigger_adam_test() {
        init();
        info!("Checking whether it still works...");
        let (mapping, ips) =
            get_test_data(Ipv4Addr::new(10, 44, 8, 92), Ipv4Addr::new(10, 44, 8, 93));
        let commands = make_commands(vec![0, 1], &mapping, &ips);
        println!("Commands are {:?}", commands);
        dispatch_adam_requests(commands);
    }
}
