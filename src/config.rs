use std::{collections::HashMap, net::Ipv4Addr};

use serde::{Deserialize, Serialize};
use super::adam::AdamCommand;
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Config {
    pub ports: Vec<VDCPPort>,
    pub adam_output_mapping:HashMap<u8,AdamCommand>,
    pub adam_ips:HashMap<u8,Ipv4Addr>
}
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self { ports: Vec::new(), adam_ips:HashMap::new(),adam_output_mapping:HashMap::new() }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VDCPPort {
    pub port: String,
    pub number:u8,
    pub name: String,
    pub segments:Vec<String>,

}
