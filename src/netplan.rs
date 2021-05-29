use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;
use std::process::Command;

#[derive(Deserialize, Serialize, Debug)]
pub struct NetplanFile {
    pub network: Network
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Network {
    pub ethernets:  HashMap<String, Ethernet>,
    pub wifis:      Option<HashMap<String, Ethernet>>,
    pub version:    u16
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Ethernet {
    pub dhcp4:          Option<bool>,
    pub optional:       Option<bool>,

    #[serde(rename(serialize = "access-points"))]
    #[serde(rename(deserialize = "access-points"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_points:  Option<HashMap<String, AccessPoint>>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AccessPoint {
    pub password: String
}

const NETPLAN_FILE: &str = r#"/etc/netplan/50-cloud-init.yaml"#;

pub fn read_netplan() -> Result<NetplanFile, String> {
    let content = match std::fs::read_to_string(PathBuf::from(NETPLAN_FILE)) {
        Ok(content) => content,
        Err(err) => return Err(err.to_string())
    };

    match serde_yaml::from_str(&content) {
        Ok(netplan_file) => Ok(netplan_file),
        Err(err) => Err(err.to_string())
    }
}

pub fn write_netplan(config: &NetplanFile) -> Result<(), String> {
    let deserialized = match serde_yaml::to_string(config) {
        Ok(string) => string,
        Err(err) => return Err(err.to_string())
    };

    let mut file = match File::create(PathBuf::from(NETPLAN_FILE)) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string())
    };

    let write_result = file.write_all(deserialized.as_bytes());
    if write_result.is_err() {
        return Err(write_result.err().unwrap().to_string());
    }

    Ok(())
}

pub fn netplan_apply() -> Result<(), String> {
    let cmd = Command::new("netplan")
        .arg("apply")
        .output();

    let output = match cmd {
        Ok(output) => output,
        Err(err) => return Err(err.to_string())
    };

    if !output.status.success() {
        let stderr = output.stderr;
        return Err(String::from_utf8(stderr).unwrap());
    }

    Ok(())
}