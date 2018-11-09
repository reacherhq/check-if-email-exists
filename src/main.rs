#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rayon;

use clap::App;
use rayon::prelude::*;
use std::process;

mod mx_hosts;
mod telnet;

fn main() {
    env_logger::init();

    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let from_email = matches.value_of("from").unwrap_or("test@example.com");
    // Calling .unwrap() is safe here because "TO" is required
    let to_email = matches.value_of("TO").unwrap();

    debug!("User inputted email {}", to_email);

    let domain = match to_email.split("@").skip(1).next() {
        Some(i) => i,
        None => {
            error!("{} is not a valid email.", to_email);
            process::exit(1);
        }
    };
    debug!("Domain name is: {}", domain);

    debug!("Getting MX lookup...");
    let hosts = mx_hosts::get_mx_lookup(domain);
    debug!("Found the following MX hosts {:?}", hosts);
    let ports = vec![25, 465, 587];
    let mut combinations = Vec::new(); // `(host, port)` combination
    for port in ports.into_iter() {
        for host in hosts.iter() {
            combinations.push((host.exchange(), port))
        }
    }

    let found = combinations
        .par_iter() // Parallelize the find_any
        .find_any(|(domain, port)| telnet::connect(from_email, to_email, domain, *port))
        .is_some();

    println!("{}", found);
}
