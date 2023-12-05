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
            Whether to check if a gravatar image is existing for the given email [env:
            CHECK_GRAVATAR=] [default: false]

        --from-email <FROM_EMAIL>
            The email to use in the `MAIL FROM:` SMTP command [env: FROM_EMAIL=] [default:
            reacher.email@gmail.com]

        --gmail-verif-method <GMAIL_VERIF_METHOD>
            Select how to verify Gmail email addresses: Api or Smtp [env: GMAIL_VERIF_METHOD=]
            [default: Smtp]

    -h, --help
            Print help information

        --haveibeenpwned-api-key <HAVEIBEENPWNED_API_KEY>
            HaveIBeenPnwed API key, ignore if not provided [env: HAVEIBEENPWNED_API_KEY=]

        --hello-name <HELLO_NAME>
            The name to use in the `EHLO:` SMTP command [env: HELLO_NAME=] [default: gmail.com]

        --hotmail-verif-method <HOTMAIL_VERIF_METHOD>
            Select how to verify Hotmail email addresses: Api, Headless or Smtp [env:
            HOTMAIL_VERIF_METHOD=] [default: Headless]

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

        --yahoo-verif-method <YAHOO_VERIF_METHOD>
            Select how to verify Yahoo email addresses: Api, Headless or Smtp [env:
            YAHOO_VERIF_METHOD=] [default: Headless]
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
