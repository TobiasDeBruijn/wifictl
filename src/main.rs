use std::process::{Command, Stdio, exit};
use crate::netplan::{read_netplan, write_netplan};
use std::collections::HashMap;

mod netplan;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut password: Option<String> = None;
    let mut ssid: Option<String> = None;

    for mut i in 0..args.len() {
        let arg = args.get(i).unwrap();
        match arg.as_str() {
            "-s" => {
                let scan_result = scan();
                if scan_result.is_err() {
                    eprintln!("{}", scan_result.err().unwrap());
                    exit(1);
                }

                for scanline in scan_result.unwrap() {
                    println!("{}", scanline);
                }

                exit(0);
            },
            "-u" => {
                i += 1;
                let ssid_local = args.get(i);
                if ssid_local.is_none() {
                    eprintln!("Missing value for '-u'");
                    exit(1);
                }

                ssid = Some(ssid_local.unwrap().clone());
            },
            "-p" => {
                i += 1;
                let password_local = args.get(i);
                if password_local.is_none() {
                    eprintln!("Missing value for '-p'");
                    exit(1);
                }

                let password_local = password_local.unwrap();
                if password_local.len() < 8 || password_local.len() > 63 {
                    eprintln!("Password invalid. Must be between 8 and 63 characters long. Inclusive.");
                    exit(1);
                }

                password = Some(password_local.clone());
            }
            _ => {}
        }
    }

    if password.is_none() || ssid.is_none() {
        eprintln!("WiFi Password or SSID not specified.");
        exit(1);
    }

    let _ = set_wifi(&ssid.unwrap(), &password.unwrap());
}

fn scan() -> Result<Vec<String>, String> {
    let iwlist_cmd = Command::new("iwlist")
        .arg("wlan0")
        .arg("scan")
        .stdout(Stdio::piped())
        .spawn();

    let iwlist_cmd = match iwlist_cmd {
        Ok(cmd) => cmd,
        Err(err) => return Err(err.to_string())
    };

    let grep_cmd = Command::new("grep")
        .arg("ESSID")
        .stdin(iwlist_cmd.stdout.unwrap())
        .output();

    let grep_cmd = match grep_cmd {
        Ok(cmd) => cmd,
        Err(err) => return Err(err.to_string())
    };

    let cmd_result = match String::from_utf8(grep_cmd.stdout) {
        Ok(result) => result,
        Err(err) => return Err(err.to_string())
    };

    let mut ssids = Vec::new();
    for line in cmd_result.lines() {
        let split: Vec<&str> = line.split(":").collect();
        let right = *split.get(1).unwrap();

        if right.eq(r#""""#) {
            continue;
        }

        let right = right.replace(r#"""#, "");
        if ssids.contains(&right) {
            continue;
        }

        ssids.push(right.to_string());
    }

    Ok(ssids)
}

fn set_wifi(ssid: &str, password: &str) -> Result<(), String> {
    let mut netplan_file = match read_netplan() {
        Ok(netplan) => netplan,
        Err(err) => {
            eprintln!("Failed to read current Netplan configuration: {}", err);
            exit(1);
        }
    };

    if netplan_file.network.wifis.is_none() {
       netplan_file.network.wifis = Some(HashMap::new());
    }

    let mut access_points = HashMap::new();
    access_points.insert(ssid.to_string(), netplan::AccessPoint { password: password.to_string() });

    let wifi = netplan::Ethernet {
        optional: Some(false),
        dhcp4: Some(true),
        access_points: Some(access_points)
    };

    let mut wifis = netplan_file.network.wifis.unwrap();
    wifis.insert("wlan0".to_string(), wifi);

    netplan_file.network.wifis = Some(wifis);

    let netplan_write_result = write_netplan(&netplan_file);
    if netplan_write_result.is_err() {
        eprintln!("Failed to write new Netplan configuration: {}", netplan_write_result.err().unwrap());
        exit(1);
    }

    let netplan_apply_result = netplan::netplan_apply();
    if netplan_apply_result.is_err() {
        eprintln!("Failed to apply Netplan configuration: {}", netplan_apply_result.err().unwrap());
        exit(1);
    }

    Ok(())
}