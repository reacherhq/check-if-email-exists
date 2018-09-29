#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use clap::App;
use std::process;

mod mx_hosts;
mod telnet;

fn main() {
    env_logger::init();

    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // Calling .unwrap() is safe here because "EMAIL" is required
    let email = matches.value_of("EMAIL").unwrap();

    debug!("Testing email {}...", email);

    let domain = match email.split("@").skip(1).next() {
        Some(i) => i,
        None => {
            error!("{} is not a valid email.", email);
            ::std::process::exit(1);
        }
    };

    debug!("Domain is: {}", domain);

    let hosts = mx_hosts::get_mx_lookup(domain);
    let ports = vec![25, 465, 587];
    for port in ports.into_iter() {
        for host in hosts.iter() {
            if telnet::connect(
                "amaury.martiny@protonmail.com",
                email,
                host.exchange(),
                port,
            ) {
                println!("true");
                process::exit(0x0100);
            };
        }
    }
    println!("false");
}
