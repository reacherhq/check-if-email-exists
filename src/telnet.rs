extern crate telnet;

use self::telnet::{Telnet, TelnetEvent};
use std::str::from_utf8;

pub fn connect() {
    let mut connection = Telnet::connect(("gmail-smtp-in.l.google.com.", 25), 256)
        .expect("Couldn't connect to the server...");

    loop {
        let event = connection.read().expect("Read Error");
        match event {
            TelnetEvent::Data(read_buffer) => {
                // Debug: print the data buffer
                println!("{}", from_utf8(&read_buffer).unwrap());

                // Buffer to write to telnet
                let write_buffer = "HELO Hi".as_bytes(); // TODO Define buffer depending on read_buffer
                connection
                    .write(&write_buffer)
                    .expect("Error while writing");
            }
            _ => {}
        }
    }
}
