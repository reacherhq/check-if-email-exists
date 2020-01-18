[![Crate](https://img.shields.io/crates/v/check-if-email-exists.svg)](https://crates.io/crates/check-if-email-exists)
[![Docs](https://docs.rs/check-if-email-exists/badge.svg)](https://docs.rs/check-if-email-exists)
[![Travis](https://img.shields.io/travis/amaurymartiny/check-if-email-exists.svg)](https://travis-ci.org/amaurymartiny/check-if-email-exists)
[![Appveyor](https://ci.appveyor.com/api/projects/status/github/amaurymartiny/check-if-email-exists?branch=master&svg=true)](https://ci.appveyor.com/project/amaurymartiny/check-if-email-exists-a08kp)
![License](https://img.shields.io/github/license/amaurymartiny/check-if-email-exists.svg)

<br /><br /><br />

<h1 align="center">check-if-email-exists</h1>
<h4 align="center">Check if an email address exists before sending the email.</h4>

<br /><br /><br />

#### 👉 Try it here: https://reacherhq.github.io

## ✅ What Does This Tool Check?

The main feature this tool checks is:

✅ **Email deliverability:** Is an email for this address deliverable?

However, it goes more into details, and checks all the following properties of an email address:

✔️ **Syntax validation.** Is the address syntactically valid?

✔️ **DNS records validation.** Does the domain of the email address have valid MX DNS records?

✔️ **Disposable email address (DEA) validation.** Is the address provided by a known [disposable email address](https://en.wikipedia.org/wiki/Disposable_email_address) provider?

✔️ **SMTP server validation.** Can the mail exchanger of the email address domain be contacted successfully?

✔️ **Mailbox disabled.** Has this email address been disabled by the email provider?

✔️ **Full inbox.** Is the inbox of this mailbox full?

✔️ **Catch-all address.** Is this email address a [catch-all](https://debounce.io/blog/help/what-is-a-catch-all-or-accept-all/) address?

Planned features:

-   [ ] **Role account validation.** Is the email address a well-known role account?
-   [ ] **Free email provider check.** Is the email address bound to a known free email provider?
-   [ ] **Syntax validation, provider-specific.** According to the syntactic rules of the target mail provider, is the address syntactically valid?
-   [ ] **Honeypot detection.** Does email address under test hide a [honeypot](https://en.wikipedia.org/wiki/Spamtrap)?
-   [ ] **Gravatar.** Does this email address have a [Gravatar](https://gravatar.com/) profile picture?

## 🤔 Why?

Many online services (https://hunter.io, http://verify-email.org, http://email-checker.net) offer this service for a paid fee. Here is an open-source alternative to those tools.

## 🚀 Try It Yourself

There are 4 ways you can try `check-if-email-exists`.

### 1. Use the Hosted Version

I created a simple static site with this tool hosted on an AWS Lambda serverless backend: http://reacherhq.github.io. The Lambda endpoint is rate-limited to prevent abuse. Also see [issue #155](https://github.com/amaurymartiny/check-if-email-exists/issues/155).

If you would like to self-host it yourself and have questions, send me a message.

### 2. Use Docker

The [Docker image](./Dockerfile) is hosted on Docker Hub: https://hub.docker.com/r/amaurymartiny/check-if-email-exists.

To run it, run the following command:

```bash
docker run -p 3000:3000 amaurymartiny/check-if-email-exists
```

You can then send a POST request with the following body (`from_email` is optional) to `http://localhost:3000`:

```json
{
	"from_email": "user@example.org",
	"to_email": "someone@gmail.com"
}
```

Here's the equivalent `curl` command:

```bash
curl -X POST -d'{"from_email":"user@example.org","to_email":"someone@gmail.com"}' http://localhost:3000
```

### 3. Download the Binary

> Note: The binary doesn't connect to any backend, it checks the email directly from your computer.

Head to the [releases page](https://github.com/amaurymartiny/check-if-email-exists/releases) and download the binary for your platform. Make sure you have [`openssl`](https://www.openssl.org/) installed on your local machine.

```
> $ check_if_email_exists --help
Check if an email address exists without sending any email.

USAGE:
    check_if_email_exists [FLAGS] [OPTIONS] [TO_EMAIL]

FLAGS:
        --http       Runs a HTTP server
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --from <FROM_EMAIL>    The from email to use in the SMTP connection [default: user@example.org]
        --http-port <PORT>     Sets the port on which the HTTP server should bind. Only used when `--http` flag is on
                               [default: 3000]

ARGS:
    <TO_EMAIL>    The email to check
```

If you run with the `--http` flag, `check-if-email-exists` will serve a HTTP server on `http://localhost:3000`. You can then send a POST request with the following body (`from_email` is optional):

```json
{
	"from_email": "user@example.org",
	"to_email": "someone@gmail.com"
}
```

Here's the equivalent `curl` command:

```bash
curl -X POST -d'{"from_email":"user@example.org","to_email":"someone@gmail.com"}' http://localhost:3000
```

**💡 PRO TIP:** To show debug logs when running the binary, run:

```bash
RUST_LOG=debug check_if_email_exists [FLAGS] [OPTIONS] [TO_EMAIL]
```

### 4. Usage as a Library (Advanced)

In your own Rust project, you can add `check-if-email-exists` in your `Cargo.toml`:

```toml
[dependencies]
check-if-email-exists = "0.6"
```

And use it in your code as follows (async/await syntax):

```rust

use check_if_email_exists::email_exists;

// First arg is the email we want to check, second arg is the FROM email used in the SMTP connection
let checked = email_exists("check.this.email@gmail.com", "user@example.org").await;

println!("{:?}", checked); // `checked` is a SingleEmail struct, see docs for more info
```

The reference docs are hosted on [docs.rs](https://docs.rs/check-if-email-exists).

## ✈️ JSON Output

The output will be a JSON with the below format, the fields should be self-explanatory. For `someone@gmail.com` (note that it is disabled by Gmail), here's the exact output:

```json
{
	"input": "someone@gmail.com",
	"misc": {
		"is_disposable": false
	},
	"mx": {
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
		"is_disabled": true
	},
	"syntax": {
		"address": "someone@gmail.com",
		"domain": "gmail.com",
		"username": "someone",
		"valid_format": true
	}
}
```

## ❓ FAQ

### The library hangs/takes a long time/doesn't show anything after 1 minute.

Most ISPs block outgoing SMTP requests through ports 25, 587 and 465, to prevent spam. `check-if-email-exists` needs to have these ports open to make a connection to the email's SMTP server, so won't work behind these ISPs, and will instead hang until it times out. There's unfortunately no easy workaround for this problem, see for example [this StackOverflow thread](https://stackoverflow.com/questions/18139102/how-to-get-around-an-isp-block-on-port-25-for-smtp). One solution is to rent a Linux cloud server with a static IP and no blocked ports.

To see in details what the binary is doing behind the scenes, run it in [verbose mode](#verbose-mode) to see the logs.

### The output shows `"connection refused"` in the `smtp` field.

This also happens when your ISP block SMTP ports, see the above answer.

## 🔨 Build From Source

First, [install Rust](https://www.rust-lang.org/tools/install); you'll need Rust 1.37.0 or later. Then, run the following commands:

```bash
# Download the code
$ git clone https://github.com/amaurymartiny/check-if-email-exists
$ cd check-if-email-exists

# Build in release mode
$ cargo build --release

# Run the binary
$ ./target/release/check_if_email_exists --help
```

## 👣 Legacy Bash Script

The 1st version of this tool was a simple bash script which made a telnet call. If you would like to use that simpler version, have a look at the [`legacy`](https://github.com/amaurymartiny/check-if-email-exists/tree/legacy) branch. The reasons for porting the bash script to the current codebase are explained [in this issue](https://github.com/amaurymartiny/check-if-email-exists/issues/4).

## 📜 License

The source code is available under the license beard dude loves. See the [LICENSE](./LICENSE) file for more info.

## 🌯 Falafel Wrap

[![Sponsor](https://img.shields.io/badge/Github%20Sponsors-%E2%9D%A4%EF%B8%8F-white)](https://github.com/sponsors/amaurymartiny/)

I don't drink coffee, but I'd enjoy a wrap from my favorite Falafel dealer. 👉 [See which one.](https://github.com/sponsors/amaurymartiny/)
