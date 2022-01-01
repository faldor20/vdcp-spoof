use std::{collections::HashMap, default, net::Ipv4Addr};

use super::adam::AdamCommand;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ports: Vec<VDCPPort>,
    pub adam_output_mapping: HashMap<u8, AdamCommand>,
    pub adam_ips: HashMap<u8, Ipv4Addr>,
    pub delays: Delays,
}
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            ports: Vec::new(),
            adam_ips: HashMap::new(),
            adam_output_mapping: HashMap::new(),
            delays: Delays::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VDCPPort {
    pub port: String,
    pub number: u8,
    pub name: String,
    pub segments: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Delays {
    pub data_read: u64,
    pub adam_command_buffer: u64,
    pub adam_pulse_off: u64,
}
impl Default for Delays {
    fn default() -> Self {
        Self {
            data_read: 10,
            adam_command_buffer: 11,
            adam_pulse_off: 20,
        }
    }
}
