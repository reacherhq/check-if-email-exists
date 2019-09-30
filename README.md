# check_if_email_exists

Check if an email address exists before sending the email.

[![](https://img.shields.io/travis/amaurymartiny/check_if_email_exists.svg)](https://travis-ci.org/amaurymartiny/check_if_email_exists)
[![](https://ci.appveyor.com/api/projects/status/github/amaurymartiny/check_if_email_exists?branch=master&svg=true)](https://ci.appveyor.com/project/amaurymartiny/check-if-email-exists-a08kp)
![License](https://img.shields.io/github/license/amaurymartiny/check_if_email_exists.svg)
[![](https://img.shields.io/badge/Buy%20me%20a%20tree-%F0%9F%8C%B3-lightgreen)](https://offset.earth/amaurymartiny)

#### 👉 Try it here: https://tinyurl.com/email-exists?to_email=EMAIL_ADDRESS

> Note: The above operation might take up to 1 minute.

> Note: For those who (judiciously) don't trust the above tinyurl link, the full URL is [this one](https://y78n51qcpj.execute-api.us-east-1.amazonaws.com/dev?to_email=EMAIL_ADDRESS).

## Why?

Many online services (https://hunter.io, http://verify-email.org, http://email-checker.net) offer this service for a paid fee. Here is an open-source alternative to those tools.

## Download the binary

Head to the [releases page](https://github.com/amaurymartiny/check_if_email_exists/releases) and download the binary for your platform.

> Note: The binary doesn't connect to the above serverless backend, it checks the mail directly from your computer.

## Usage

Make sure you have [`openssl`](https://www.openssl.org/) installed.

```
USAGE:
    check_if_email_exists [OPTIONS] <TO_EMAIL>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --from <FROM_EMAIL>    The from email to use in the SMTP connection [default: user@example.org]

ARGS:
    <TO_EMAIL>    The email to check
```

The output will be a JSON with the following format, for `someone@gmail.com` (note that it is disabled by Gmail):

```json
{
	"mx": {
		"is_disposable": false,
		"records": [
			"alt3.gmail-smtp-in.l.google.com.",
			"gmail-smtp-in.l.google.com.",
			"alt1.gmail-smtp-in.l.google.com.",
			"alt4.gmail-smtp-in.l.google.com.",
			"alt2.gmail-smtp-in.l.google.com."
		]
	},
	"smtp": {
		"has_full_inbox": false,
		"is_catch_all": false,
		"is_deliverable": false,
		"is_disabled": true,
	},
	"syntax": {
		"address": "someone@gmail.com",
		"domain": "gmail.com",
		"username": "someone",
		"valid_format": true
	}
}
```

### Verbose Mode

To show debug logs when running the binary, run:

```bash
RUST_LOG=debug check_if_email_exists [OPTIONS] <TO_EMAIL>
```

## FAQ

### The binary hangs/takes a long time/doesn't show anything after 1 minute.

Most ISPs block outgoing SMTP requests through ports 25, 587 and 465, to prevent spam. `check_if_email_exists` needs to have these ports open to make a connection to the email's SMTP server, so won't work behind these ISPs, and will instead hang until it times out. There's unfortunately no easy workaround for this problem, see for example [this StackOverflow thread](https://stackoverflow.com/questions/18139102/how-to-get-around-an-isp-block-on-port-25-for-smtp). One solution is to rent a Linux cloud server with a static IP and no blocked ports.

To see in details what the binary is doing behind the scenes, run it in [verbose mode](#verbose-mode) to see the logs.

## Legacy Bash Script

The 1st version of this tool was a simple bash script which made a telnet call. If you would like to use that simpler version, have a look at the [`legacy`](https://github.com/amaurymartiny/check_if_email_exists/tree/legacy) branch. The reasons for porting the bash script to the current codebase are explained [in this issue](https://github.com/amaurymartiny/check_if_email_exists/issues/4).

## Build From Source

First, [install Rust](https://www.rust-lang.org/tools/install); you'll need Rust 1.37.0 or later. Then, clone the source code locally:

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

GPL-3.0. See the [LICENSE](./LICENSE) file for more info.
