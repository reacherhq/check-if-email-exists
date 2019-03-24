# check_if_email_exists

Check if an email address exists before sending the email.

[![](https://img.shields.io/travis/amaurymartiny/check_if_email_exists.svg)](https://travis-ci.org/amaurymartiny/check_if_email_exists)
[![](https://ci.appveyor.com/api/projects/status/github/amaurymartiny/check_if_email_exists?branch=master&svg=true)](https://ci.appveyor.com/project/amaurymartiny/check-if-email-exists-a08kp)

## Why?

Many online services (https://hunter.io, http://verify-email.org, http://email-checker.net) offer this service for a paid fee. Here is an open-source alternative to those tools.

## Download the binary

Head to the [releases page](https://github.com/amaurymartiny/check_if_email_exists/releases) and download the binary for your platform.

## Usage

Make sure you have [`openssl`](https://www.openssl.org/) installed.

```
USAGE:
    check_if_email_exists [OPTIONS] <TO_EMAIL>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --from <FROM_EMAIL>    The from email to use in the SMTP connection (default: test@example.com)

ARGS:
    <TO_EMAIL>    The email to check
```

### Verbose Mode

To show debug logs when running the binary, run:

```bash
RUST_LOG=debug check_if_email_exists [OPTIONS] <TO_EMAIL>
```

## FAQ

### The binary hangs/takes a long time/doesn't show anything.

Most ISPs block outgoing SMTP requests through ports 25, 587 and 465, to prevent spam. `check_if_email_exists` needs to have these ports open to make a connection to the email's SMTP server, so won't work behind these ISPs, and will instead hang until it times out. There's unfortunately no easy workaround for this problem, see for example [this StackOverflow thread](https://stackoverflow.com/questions/18139102/how-to-get-around-an-isp-block-on-port-25-for-smtp). One solution is to rent a Linux cloud server with a static IP and no blocked ports.

## Legacy Bash Script

The 1st version of this tool was a simple bash script which made a telnet call. If you would like to use that simpler version, have a look at the [`legacy`](https://github.com/amaurymartiny/check_if_email_exists/tree/legacy) branch. The reasons for porting the bash script to the current codebase are explained [here](https://github.com/amaurymartiny/check_if_email_exists/issues/4).

## Build From Source

First, [install Rust](https://www.rust-lang.org/tools/install). Then, clone the source code locally:

```bash
# Download the code
$ git clone https://github.com/amaurymartiny/check_if_email_exists
$ cd check_if_email_exists

# Build in release mode
$ cargo build --release

# Run the binary
$ ./target/release/check_if_email_exists --help
```

## License

See the LICENSE file.
