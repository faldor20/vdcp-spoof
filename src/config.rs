use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Config {
    pub ports: Vec<VDCPPort>,
}
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self { ports: Vec::new() }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VDCPPort {
    pub port: String,
    pub number:u8,
    pub name: String,
    pub segments:Vec<String>
}
