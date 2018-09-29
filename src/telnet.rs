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

pub fn connect(from: &str, to: &str, domain: &Name, port: u16) -> bool {
    debug!("Telnet connection to {}...", domain);

    let mut connection = match Telnet::connect((domain.to_utf8().as_str(), port), 256) {
        Ok(i) => i,
        Err(e) => {
            debug!("Failed to connect, {}", e);
            return false;
        }
    };

    let mut step = Step::Welcome;

    loop {
        match step {
            Step::Welcome | Step::SentHelo | Step::SentMailFrom | Step::SentRcptTo => {
                // If we are in one of the above steps, then we read from the
                // telnet connection.
                let event = match connection.read() {
                    Ok(i) => i,
                    Err(_) => break,
                };

                match event {
                    TelnetEvent::Data(read_buffer) => {
                        // `answer` is what we get from the server
                        let answer = match from_utf8(&read_buffer) {
                            Ok(i) => i,
                            _ => break,
                        };

                        debug!("Received: {}", answer);

                        // `question` is what we ask the server
                        let mut question = match step {
                            Step::Welcome => {
                                if answer.contains("220") {
                                    step = Step::SentHelo;
                                    String::from("HELO Hi")
                                } else {
                                    break;
                                }
                            }
                            Step::SentHelo => {
                                if answer.contains("250") {
                                    step = Step::SentMailFrom;
                                    format!("{}{}{}", "MAIL FROM: <", from, ">")
                                } else {
                                    break;
                                }
                            }
                            Step::SentMailFrom => {
                                if answer.contains("2.1.0") {
                                    step = Step::SentRcptTo;
                                    format!("{}{}{}", "RCPT TO: <", to, ">")
                                } else {
                                    break;
                                }
                            }
                            Step::SentRcptTo => {
                                // 2.1.5 means address exists
                                if answer.contains("2.1.5") {
                                    step = Step::Found;
                                } else {
                                    step = Step::NotFound;
                                }
                                String::from("QUIT")
                            }
                            _ => panic!("Step is Found/NotFound where it shouldn't be."),
                        };

                        debug!("Sent: {}", question);

                        // Buffer to write to telnet
                        question.push_str("\n");
                        let write_buffer = question.as_bytes();
                        if let Err(e) = connection.write(&write_buffer) {
                            debug!("Error while writing, {}", e);
                        }
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }

    match step {
        Step::Found => true,
        _ => false,
    }
}
