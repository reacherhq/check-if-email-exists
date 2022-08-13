[![Crate](https://img.shields.io/crates/v/check-if-email-exists.svg)](https://crates.io/crates/check-if-email-exists)
[![Docs](https://docs.rs/check-if-email-exists/badge.svg)](https://docs.rs/check-if-email-exists)
[![Actions Status](https://github.com/reacherhq/check-if-email-exists/workflows/CI/badge.svg)](https://github.com/reacherhq/check-if-email-exists/actions)
[![Github Sponsor](https://img.shields.io/static/v1?label=Sponsor&message=%E2%9D%A4&logo=GitHub&link=https://github.com/sponsors/amaurym)](https://github.com/sponsors/amaurym)

<br /><br /><br />

<h1 align="center">check-if-email-exists</h1>
<h4 align="center">Check if an email address exists without sending any email.</h4>

<br /><br /><br />

## 👉 Live Demo: https://reacher.email

<a href="https://www.hookdoo.com/?github"><img src="https://storage.googleapis.com/saasify-uploads-prod/696e287ad79f0e0352bc201b36d701849f7d55e7.svg" height="96" alt="hookdoo" align="left" /></a>

If you don't have time to waste configuring, hosting, debugging, and maintaining your own email verifier, we offer a **SaaS** solution that has all of the capabilities `check-if-email-exists` provides, plus a lot more, and all that packaged in a nice friendly web interface. If you are interested, find out more at [Reacher](https://reacher.email/?ref=github). If you have any questions, you can contact me at amaury@reacher.email.

<br />

## What Does This Tool Check?

| Included? | Feature                                       | Description                                                                                                                     | JSON field                                                                  |
| --------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| ✅         | **Email reachability**                        | How confident are we in sending an email to this address? Can be one of `safe`, `risky`, `invalid` or `unknown`.                | `is_reachable`                                                              |
| ✅         | **Syntax validation**                         | Is the address syntactically valid?                                                                                             | `syntax.is_valid_syntax`                                                    |
| ✅         | **DNS records validation**                    | Does the domain of the email address have valid MX DNS records?                                                                 | `mx.accepts_mail`                                                           |
| ✅         | **Disposable email address (DEA) validation** | Is the address provided by a known [disposable email address](https://en.wikipedia.org/wiki/Disposable_email_address) provider? | `misc.is_disposable`                                                        |
| ✅         | **SMTP server validation**                    | Can the mail exchanger of the email address domain be contacted successfully?                                                   | `smtp.can_connect_smtp`                                                     |
| ✅         | **Email deliverability**                      | Is an email sent to this address deliverable?                                                                                   | `smtp.is_deliverable`                                                       |
| ✅         | **Mailbox disabled**                          | Has this email address been disabled by the email provider?                                                                     | `smtp.is_disabled`                                                          |
| ✅         | **Full inbox**                                | Is the inbox of this mailbox full?                                                                                              | `smtp.has_full_inbox`                                                       |
| ✅         | **Catch-all address**                         | Is this email address a [catch-all](https://debounce.io/blog/help/what-is-a-catch-all-or-accept-all/) address?                  | `smtp.is_catch_all`                                                         |
| ✅         | **Role account validation**                   | Is the email address a well-known role account?                                                                                 | `misc.is_role_account`                                                      |
| 🔜         | **Free email provider check**                 | Is the email address bound to a known free email provider?                                                                      | [Issue #89](https://github.com/reacherhq/check-if-email-exists/issues/89)   |
| 🔜         | **Syntax validation, provider-specific**      | According to the syntactic rules of the target mail provider, is the address syntactically valid?                               | [Issue #90](https://github.com/reacherhq/check-if-email-exists/issues/90)   |
| 🔜         | **Honeypot detection**                        | Does email address under test hide a [honeypot](https://en.wikipedia.org/wiki/Spamtrap)?                                        | [Issue #91](https://github.com/reacherhq/check-if-email-exists/issues/91)   |
| 🔜         | **Gravatar**                                  | Does this email address have a [Gravatar](https://gravatar.com/) profile picture?                                               | [Issue #92](https://github.com/reacherhq/check-if-email-exists/issues/92)   |
| 🔜         | **Have I Been Pwned?**                        | Has this email been compromised in a [data breach](https://haveibeenpwned.com/)?                                                | [Issue #289](https://github.com/reacherhq/check-if-email-exists/issues/289) |

## 🤔 Why?

Many online services (https://hunter.io, https://verify-email.org, https://email-checker.net) offer this service for a paid fee. Here is an open-source alternative to those tools.

## License

`check-if-email-exists`'s source code is provided under a **dual license model**.

### Commercial license

If you want to use `check-if-email-exists` to develop commercial sites, tools, and applications, the Commercial License is the appropriate license. With this option, your source code is kept proprietary. Purchase a `check-if-email-exists` Commercial License at https://reacher.email/pricing.

### Open source license

If you are creating an open-source application under a license compatible with the GNU Affero GPL license v3, you may use `check-if-email-exists` under the terms of the [AGPL-3.0](./LICENSE.AGPL).

[Read more](https://help.reacher.email/reacher-licenses) about Reacher's license.

## Try It Yourself

There are 5 ways you can try `check-if-email-exists`.

### 1. Use the Hosted Version: https://reacher.email 🥇

Reacher is a simple SaaS using this library, also [open-source](https://github.com/reacherhq/backend)!

> If you would like a high free tier to test Reacher, consider [sponsoring me](https://github.com/sponsors/amaurym/)! You'll get 8000 free email verifications every month, and this contribution would mean A WHOLE LOT to me.

### 2. One-Click Deploy to Heroku

Reacher provides a fully-fledged REST backend at https://github.com/reacherhq/backend. It is the same backend running for our main product https://reacher.email.

The backend is built using the fast web framework [warp](https://github.com/seanmonstar/warp) and exposes an API endpoint for making email verifications.

For a one-click deploy to [Heroku](https://heroku.com), click on the purple Heroku button at [reacherhq/backend](https://github.com/reacherhq/backend#get-started).

### 3. Use Docker

A Docker image with a fully-fledged HTTP backend is hosted on https://hub.docker.com/r/reacherhq/backend.

For more information on how to use the Docker image, please head to [reacherhq/backend](https://github.com/reacherhq/backend#2-use-docker).

### 4. Download the Binary

> Note: The binary doesn't connect to any backend, it checks the email directly from your computer.

Head to the [releases page](https://github.com/reacherhq/check-if-email-exists/releases) and download the binary for your platform.

```
> $ check_if_email_exists --help
check_if_email_exists 0.8.32
Check if an email address exists without sending an email.

USAGE:
    check_if_email_exists [FLAGS] [OPTIONS] [TO_EMAIL]

ARGS:
    <TO_EMAIL>    The email to check

FLAGS:
    -h, --help       Print help information
        --http       DEPRECATED. Runs an HTTP server. This option will be removed in v0.9.0
    -V, --version    Print version information

OPTIONS:
        --from-email <FROM_EMAIL>
            The email to use in the `MAIL FROM:` SMTP command [env: FROM_EMAIL=] [default:
            user@example.org]

        --hello-name <HELLO_NAME>
            The name to use in the `EHLO:` SMTP command [env: HELLO_NAME=] [default: localhost]

        --http-host <HTTP_HOST>
            DEPRECATED. Sets the host IP address on which the HTTP server should bind. Only used
            when `--http` flag is on. This option will be removed in v0.9.0 [env: HOST=] [default:
            127.0.0.1]

        --http-port <HTTP_PORT>
            DEPRECATED. Sets the port on which the HTTP server should bind. Only used when the `--http`
            flag is on. If not set, then it will use $PORT or default to 3000. This option will be
            removed in v0.9.0 [env: PORT=] [default: 3000]

        --proxy-host <PROXY_HOST>
            Use the specified SOCKS5 proxy host to perform email verification [env: PROXY_HOST=]

        --proxy-port <PROXY_PORT>
            Use the specified SOCKS5 proxy port to perform email verification. Only used when
            `--proxy-host` flag is set [env: PROXY_PORT=] [default: 1080]

        --smtp-port <SMTP_PORT>
            The email to check [env: SMTP_PORT=] [default: 25]

        --yahoo-use-api <YAHOO_USE_API>
            For Yahoo email addresses, use Yahoo's API instead of connecting directly to their SMTP
            servers [env: YAHOO_USE_API=] [default: true]
```

**💡 PRO TIP:** To show debug logs when running the binary, run:

```bash
RUST_LOG=debug check_if_email_exists [FLAGS] [OPTIONS] [TO_EMAIL]
```

### 5. Usage as a Rust Library

In your own Rust project, you can add `check-if-email-exists` in your `Cargo.toml`:

```toml
[dependencies]
check-if-email-exists = "0.8"
```

And use it in your code as follows:

```rust
use check_if_email_exists::{check_email, CheckEmailInput, CheckEmailInputProxy};

async fn check() {
    // Let's say we want to test the deliverability of someone@gmail.com.
    let mut input = CheckEmailInput::new(vec!["someone@gmail.com".into()]);

    // Optionally, we can also tweak the configuration parameters used in the
    // verification.
    input
        .set_from_email("me@example.org".into()) // Used in the `MAIL FROM:` command
        .set_hello_name("example.org".into())    // Used in the `EHLO` command
        .set_proxy(CheckEmailInputProxy {        // Use a SOCKS5 proxy to verify the email
            host: "my-proxy.io".into(),
            port: 1080,
            username: None,                      // You can also set it non-empty
            password: None
        });

    // Verify this email, using async/await syntax.
    let result = check_email(&input).await;

    // `result` is a `Vec<CheckEmailOutput>`, where the CheckEmailOutput
    // struct contains all information about our email.
    println!("{:?}", result);
}
```

The reference docs are hosted on [docs.rs](https://docs.rs/check-if-email-exists).

## ✈️ JSON Output

The output will be a JSON with the below format, the fields should be self-explanatory. For `someone@gmail.com` (note that it is disabled by Gmail), here's the exact output:

```json
{
	"input": "someone@gmail.com",
	"is_reachable": "invalid",
	"misc": {
		"is_disposable": false,
		"is_role_account": false
	},
	"mx": {
		"accepts_mail": true,
		"records": [
			"alt3.gmail-smtp-in.l.google.com.",
			"gmail-smtp-in.l.google.com.",
			"alt1.gmail-smtp-in.l.google.com.",
			"alt4.gmail-smtp-in.l.google.com.",
			"alt2.gmail-smtp-in.l.google.com."
		]
	},
	"smtp": {
		"can_connect_smtp": true,
		"has_full_inbox": false,
		"is_catch_all": false,
		"is_deliverable": false,
		"is_disabled": true
	},
	"syntax": {
		"domain": "gmail.com",
		"is_valid_syntax": true,
		"username": "someone"
	}
}
```

You can also take a look at the [documentation](https://help.reacher.email/rest-api-documentation) of this JSON object.

## ❓ FAQ

#### What does `is_reachable: "unknown"` mean?

This means that the server does not allow real-time verification of an email right now. It may happen for multiple reasons: your IP is blacklisted, the SMTP port 25 is blocked, the email account is momentarily receiving too many emails (spam protection)... or the email provider simply does not allow real-time verification at all. The details of this `"unknown"` case can be found in the `smtp.error` and `mx.error` fields.

#### The library hangs/takes a long time/doesn't show anything after 1 minute.

Most ISPs block outgoing SMTP requests through port 25, to prevent spam. `check-if-email-exists` needs to have this port open to make a connection to the email's SMTP server, so won't work behind these ISPs, and will instead hang until it times out. There's unfortunately no easy workaround for this problem, see for example [this StackOverflow thread](https://stackoverflow.com/questions/18139102/how-to-get-around-an-isp-block-on-port-25-for-smtp). One solution is to rent a Linux cloud server with a static IP and no blocked ports, see for example our [Deploy to Heroku](#2-deploy-to-heroku) section.

To see in detail what the binary is doing behind the scenes, run it in verbose mode to see the logs.

#### I have another question

Feel free to check out Reacher's [FAQ](https://help.reacher.email/faq).

## 🔨 Build From Source

First, [install Rust](https://www.rust-lang.org/tools/install); you'll need Rust 1.37.0 or later. Then, run the following commands:

```bash
# Download the code
$ git clone https://github.com/reacherhq/check-if-email-exists
$ cd check-if-email-exists

# Build in release mode
$ cargo build --release

# Run the binary
$ ./target/release/check_if_email_exists --help
```

## Legacy Bash Script

The 1st version of this tool was a simple bash script that made a telnet call. If you would like to use that simpler version, have a look at the [`legacy`](https://github.com/reacherhq/check-if-email-exists/tree/legacy) branch. The reasons for porting the bash script to the current codebase are explained [in issue #4](https://github.com/reacherhq/check-if-email-exists/issues/4).
