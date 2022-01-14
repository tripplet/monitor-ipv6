#![feature(ip)]

use clap::Parser;
use get_if_addrs::{get_if_addrs, IfAddr, Interface};
use log::{debug, info, error};

use std::process::Command;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

// The main config
#[derive(Debug, Parser)]
#[clap(version)]
struct Config {
    /// Network interface to check
    #[clap(short, long, env)]
    interface: String,

    /// Intervall (in seconds) to check the interface
    #[clap(long, default_value = "10", env)]
    intervall: u16,

    /// Intervall (in seconds) to check the interface
    #[clap(long, default_value="info", parse(try_from_str = log::Level::from_str), env)]
    log_level: log::Level,
}

fn main() {
    // Parse arguments
    let cfg = Config::parse();

    // Initialize logger
    simple_logger::init_with_level(cfg.log_level).unwrap();

    loop {
        let global_v6_addresses = get_global_v6_addresses(&cfg.interface);

        if let Err(err) = global_v6_addresses {
            error!("Error getting interface info: {}", err);
        } else if let Ok(addresses) = global_v6_addresses {
            if addresses.is_empty() {
                let status = Command::new("/usr/bin/networkctl")
                    .arg("reconfigure")
                    .arg(&cfg.interface)
                    .status();

                info!("No global ipv6 address found, reconfiguring {}", match status {
                    Err(err) => format!("failed: {}", err),
                    Ok(exit_status) if exit_status.success() => format!("successful"),
                    Ok(exit_status) => format!("not successful, return code: {:?}", exit_status.code())
                });

                thread::sleep(Duration::from_secs(5));
                continue;
            } else {
                debug!("Global ipv6 addresses: {:?}", addresses);
            }
        }

        thread::sleep(Duration::from_secs(cfg.intervall.into()));
    }
}

fn get_global_v6_addresses(interface: &str) -> Result<Vec<std::net::Ipv6Addr>, std::io::Error> {
    Ok(get_if_addrs()?
        .iter()
        .filter_map(|inf| match inf {
            Interface {
                name,
                addr: IfAddr::V6(addr),
            } if *name == interface && !addr.is_loopback() && addr.ip.is_global() => Some(addr.ip),
            _ => None,
        })
        .collect())
}
