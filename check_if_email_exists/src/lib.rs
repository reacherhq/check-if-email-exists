extern crate lettre;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate rayon;
extern crate trust_dns_resolver;

use lettre::smtp::{SMTP_PORT, SUBMISSIONS_PORT, SUBMISSION_PORT};
use rayon::prelude::*;
use std::process;

mod mx_hosts;
mod smtp;

pub fn email_exists(from_email: &str, to_email: &str) -> Option<bool> {
	debug!("Checking email '{}'", to_email);

	let domain = match to_email.split("@").skip(1).next() {
		Some(i) => i,
		None => {
			error!("'{}' is not a valid email.", to_email);
			process::exit(1);
		}
	};
	debug!("Domain name is '{}'", domain);

	debug!("Getting MX lookup...");
	let hosts = mx_hosts::get_mx_lookup(domain);
	debug!("Found the following MX hosts {:?}", hosts);
	let ports = vec![SMTP_PORT, SUBMISSION_PORT, SUBMISSIONS_PORT]; // [25, 587, 465]
	let mut combinations = Vec::new(); // `(host, port)` combination
	for port in ports.into_iter() {
		for host in hosts.iter() {
			combinations.push((host.exchange(), port))
		}
	}

	combinations
		// Parallely find any combination that returns true for email_exists
		.par_iter()
		.flat_map(|(host, port)| smtp::email_exists(from_email, to_email, host, *port))
		.find_any(|_| true)
}
