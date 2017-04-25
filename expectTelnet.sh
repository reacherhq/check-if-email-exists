#! /usr/bin/env expect

# Retrive arguments
set email [lindex $argv 0]
set host [lindex $argv 1]
set port [lindex $argv 2]

# Start a Telnet communication
spawn telnet $host $port;
expect "220"
send "MAIL FROM: test@test.org\r";
expect "250"
send "RCPT TO: $email\r";
expect "*"
send "QUIT\r"
expect eof
