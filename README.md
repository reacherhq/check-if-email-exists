# check_if_email_exists

Check if an email address exists before sending the email.

[![](https://img.shields.io/travis/amaurymartiny/check_if_email_exists.svg)](https://travis-ci.org/amaurymartiny/check-if-email-exists/)
[![](https://img.shields.io/appveyor/ci/amaurymartiny/check-if-email-exists-a08kp.svg)](https://ci.appveyor.com/project/amaurymartiny/check_if_email_exists-a08kp)

## Why?

Many online services (https://hunter.io, http://verify-email.org, http://email-checker.net) offer this service for a paid fee. Here is an open-source alternative to those tools.

## Download the binary

Head to the [releases page](https://github.com/amaurymartiny/check_if_email_exists/releases) and download the binary for your platform.

## Usage

Make sure you have [`openssl`](https://www.openssl.org/) installed.

```bash
USAGE:
    check_if_email_exists [OPTIONS] <TO_EMAIL>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --from <FROM_EMAIL>    The from email to use in the Telnet connection (default: test@example.com)

ARGS:
    <TO_EMAIL>    The email to check
```

## Legacy Bash Script

The 1st version of this tool was a simple bash script which made a telnet call. If you would like to use that simpler version, have a look at the [`legacy`](https://github.com/amaurymartiny/check_if_email_exists/tree/legacy) branch. The reasons for porting the bash script to the current codebase are explained [here](https://github.com/amaurymartiny/check_if_email_exists/issues/4).

## Contribute

### Install Rust

check_if_email_exists requires **latest stable Rust version** to build.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have `rustup`, you can install it like this:

-   Linux:

    ```bash
    $ curl https://sh.rustup.rs -sSf | sh
    ```

    check_if_email_exists also requires `gcc`, `g++`, `libudev-dev`, `pkg-config`, `file`, `make`, and `cmake` packages to be installed.

-   OSX:

    ```bash
    $ curl https://sh.rustup.rs -sSf | sh
    ```

    `clang` is required. It comes with Xcode command line tools or can be installed with homebrew.

-   Windows
    Make sure you have Visual Studio 2015 with C++ support installed. Next, download and run the `rustup` installer from
    https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe, start "VS2015 x64 Native Tools Command Prompt", and use the following command to install and set up the `msvc` toolchain:
    ```bash
    $ rustup default stable-x86_64-pc-windows-msvc
    ```

### Clone the source code locally

```bash
# Download the code
$ git clone https://github.com/amaurymartiny/check_if_email_exists
$ cd check_if_email_exists

# Make your modifications in src/

# Build
$ cargo build

# Run the binary
$ ./target/debug/check_if_email_exists --help
```

## License

See the LICENSE file.
