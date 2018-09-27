extern crate telnet;
extern crate trust_dns_resolver;

use self::telnet::{Telnet, TelnetEvent};
use self::trust_dns_resolver::Name;
use std::str::from_utf8;

// The step into the telnet discussion we're in.
enum Step {
    Welcome,
    SentHelo,
    SentMailFrom,
    SentRcptTo,
    Found,
    NotFound,
}

pub fn connect(from: &str, to: &str, domain: Name, port: u16) {
    debug!("Connecting to: {}...", domain);

    let mut connection =
        Telnet::connect((domain, port), 256).expect("Couldn't connect to the server...");

    let mut step = Step::Welcome;
    loop {
        let event = connection.read().expect("Read Error");
        match event {
            TelnetEvent::Data(read_buffer) => {
                // `answer` is what we get from the server
                let answer = from_utf8(&read_buffer).unwrap();

                debug!("Received: {}", answer);

                // `question` is what we ask the server
                let question = match step {
                    Step::Welcome => {
                        if answer.contains("220") {
                            step = Step::SentHelo;
                            String::from("HELO Hi\n")
                        } else {
                            panic!("Got an unexpected answer at Welcome step.");
                        }
                    }
                    Step::SentHelo => {
                        if answer.contains("250") {
                            step = Step::SentMailFrom;
                            format!("{}{}{}", "MAIL FROM: <", from, ">\n")
                        } else {
                            panic!("Got an unexpected answer at SentHelo step.");
                        }
                    }
                    Step::SentMailFrom => {
                        if answer.contains("250") {
                            step = Step::SentRcptTo;
                            format!("{}{}{}", "RCPT TO: <", to, ">\n")
                        } else {
                            panic!("Got an unexpected answer at SentMailFrom step.");
                        }
                    }
                    Step::SentRcptTo => {
                        // 2.1.5 means address exists
                        if answer.contains("2.1.5") {
                            step = Step::Found;
                        } else {
                            step = Step::NotFound;
                        }
                        String::from("QUIT\n")
                    }
                    _ => panic!("Got an unexpected answer at Found/NotFound step."),
                };

                debug!("Sent: {}", question);

                // Buffer to write to telnet
                let write_buffer = question.as_bytes(); // TODO Define buffer depending on read_buffer
                connection
                    .write(&write_buffer)
                    .expect("Error while writing");
            }
            _ => {}
        }
    }
}
