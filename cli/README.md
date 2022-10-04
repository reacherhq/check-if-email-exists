<br /><br /><br />

<h1 align="center">check-if-email-exists CLI</h1>
<h4 align="center">Email verification from your terminal.</h4>

<br /><br /><br />

> Note: The CLI binary doesn't connect to any backend, it checks the email directly from your computer.

## Usage

Head to the [releases page](https://github.com/reacherhq/check-if-email-exists/releases) and download the binary for your platform.

Then run:

```bash
> $ check_if_email_exists --help
check-if-email-exists-cli 0.9.1
Check if an email address exists without sending any email.

USAGE:
    check_if_email_exists [OPTIONS] <TO_EMAIL>

ARGS:
    <TO_EMAIL>    The email to check

OPTIONS:
        --check-gravatar <CHECK_GRAVATAR>
            Whether to check for an existing gravatar image [env: CHECK_GRAVATAR=] [default: false]

        --from-email <FROM_EMAIL>
            The email to use in the `MAIL FROM:` SMTP command [env: FROM_EMAIL=] [default:
            user@example.org]

    -h, --help
            Print help information

        --hello-name <HELLO_NAME>
            The name to use in the `EHLO:` SMTP command [env: HELLO_NAME=] [default: localhost]

        --proxy-host <PROXY_HOST>
            Use the specified SOCKS5 proxy host to perform email verification [env: PROXY_HOST=]

        --proxy-password <PROXY_PASSWORD>
            Username passed to the specified SOCKS5 proxy port to perform email verification. Only
            used when `--proxy-host` flag is set [env: PROXY_PASSWORD=]

        --proxy-port <PROXY_PORT>
            Use the specified SOCKS5 proxy port to perform email verification. Only used when
            `--proxy-host` flag is set [env: PROXY_PORT=] [default: 1080]

        --proxy-username <PROXY_USERNAME>
            Username passed to the specified SOCKS5 proxy port to perform email verification. Only
            used when `--proxy-host` flag is set [env: PROXY_USERNAME=]

        --smtp-port <SMTP_PORT>
            The port to use for the SMTP request [env: SMTP_PORT=] [default: 25]

    -V, --version
            Print version information

        --yahoo-use-api <YAHOO_USE_API>
            For Yahoo email addresses, use Yahoo's API instead of connecting directly to their SMTP
            servers [env: YAHOO_USE_API=] [default: true]

        --gmail-use-api <GMAIL_USE_API>
            For Gmail email addresses, use Gmail's API instead of connecting directly to their SMTP
            servers [env: GMAIL_USE_API=] [default: false]
```

**💡 PRO TIP:** To show debug logs when running the binary, run:

```bash
RUST_LOG=debug check_if_email_exists
```

## Build From Source

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
