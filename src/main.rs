#![feature(ip)]

use clap::Parser;
use dbus::blocking::Connection;
use get_if_addrs::{get_if_addrs, IfAddr, Interface};
use log::{debug, error, info};

use std::cmp;
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

    /// Time to wait before first check (in seconds)
    #[clap(long, default_value = "300", env)]
    startup_delay: u16,
}

fn main() {
    // Parse arguments
    let cfg = Config::parse();

    // Initialize logger
    simple_logger::init_with_level(cfg.log_level).unwrap();

    if cfg.startup_delay > 0 {
        info!("Waiting startup delay of {} seconds", cfg.startup_delay);
        thread::sleep(Duration::from_secs(cfg.startup_delay.into()));
        info!("Starting check loop");
    }

    let mut exponential_backoff = 0u8;
    loop {
        thread::sleep(Duration::from_secs(u64::pow(2, exponential_backoff as u32)));

        let global_v6_addresses = get_global_v6_addresses(&cfg.interface);

        if let Err(err) = global_v6_addresses {
            error!("Error getting interface info: {err}");
        } else if let Ok(addresses) = global_v6_addresses {
            if addresses.is_empty() {
                info!(
                    "No global ipv6 address found, reconfiguring {}",
                    match reconfigure_interface(&cfg.interface) {
                        Err(err) => format!("failed: {}", err),
                        Ok(_) => "successful".to_string(),
                    }
                );

                // wait up to 2**9 = 512s (8m32s) in case of continous reconfigures
                exponential_backoff = cmp::min(exponential_backoff + 1, 9);

                thread::sleep(Duration::from_secs(5));
                continue;
            } else {
                exponential_backoff = 0;
                debug!("Global ipv6 addresses: {addresses:?}");
            }
        }

        thread::sleep(Duration::from_secs(cfg.intervall.into()));
    }
}

fn reconfigure_interface(interface: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.freedesktop.network1",
        "/org/freedesktop/network1",
        Duration::from_millis(5000),
    );

    let (index,): (i32,) = proxy.method_call(
        "org.freedesktop.network1.Manager",
        "GetLinkByName",
        (interface,),
    )?;
    
    Ok(proxy.method_call(
        "org.freedesktop.network1.Manager",
        "ReconfigureLink",
        (index,),
    )?)
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
