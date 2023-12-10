[![Docker](https://img.shields.io/docker/v/reacherhq/backend?color=0db7ed&label=docker&sort=date)](https://hub.docker.com/r/reacherhq/backend)
[![Actions Status](https://github.com/reacherhq/check-if-email-exists/workflows/pr/badge.svg)](https://github.com/reacherhq/check-if-email-exists/actions)

<br /><br />

<p align="center"><img align="center" src="https://storage.googleapis.com/saasify-uploads-prod/696e287ad79f0e0352bc201b36d701849f7d55e7.svg" height="96" alt="reacher" /></p>
<h1 align="center">⚙️ Reacher Backend</h1>
<h4 align="center">REST Server for Reacher Email Verification API: https://reacher.email.</h4>

<br /><br />

This crate holds the backend for [Reacher](https://reacher.email). The backend is a HTTP server with the following components:

-   [`check-if-email-exists`](https://github.com/reacherhq/check-if-email-exists), which performs the core email verification logic,
-   [`warp`](https://github.com/seanmonstar/warp) web framework.

> 💡 Update: Bulk email verification is currently being worked on, and is in **beta** phase. You can try to use the bulk verification endpoints by setting the environment variable `RCH_ENABLE_BULK=1`. The documentation is currently being [worked on](https://github.com/reacherhq/backend/issues/321).

## Get Started

The [Docker image](./Dockerfile) is hosted on Docker Hub: https://hub.docker.com/r/reacherhq/backend.

To run it, run the following command:

```bash
docker run -p 8080:8080 reacherhq/backend:latest
```

Then send a `POST http://localhost:8080/v0/check_email` request with the following body:

```js
{
    "to_email": "someone@gmail.com",
    "from_email": "my@my-server.com", // (optional) email to use in the `FROM` SMTP command, defaults to "user@example.org"
    "hello_name": "my-server.com",    // (optional) name to use in the `EHLO` SMTP command, defaults to "localhost"
    "proxy": {                        // (optional) SOCK5 proxy to run the verification through, default is empty
        "host": "my-proxy.io",
        "port": 1080,
        "username": "me",             // (optional) Proxy username
        "password": "pass"            // (optional) Proxy password
    },
    "smtp_port": 587                  // (optional) SMTP port to do the email verification, defaults to 25
}
```

### Configuration

These are the environment variables used to configure the HTTP server. To pass them to the Docker container, use the `-e {ENV_VAR}={VALUE}` flag.

| Env Var                             | Required?                   | Description                                                                                                                                                                                                                                 | Default                 |
| ----------------------------------- | --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| `RUST_LOG`                          | No                          | One of `trace,debug,warn,error,info`. 💡 PRO TIP: `RUST_LOG=debug` is very handful for debugging purposes.                                                                                                                                  | not defined             |
| `RCH_ENABLE_BULK`                   | No                          | If set to `1`, then bulk verification endpoints will be added to the backend.                                                                                                                                                               | 0                       |
| `DATABASE_URL`                      | Yes if `RCH_ENABLE_BULK==1` | [Bulk] Database connection string for storing results and task queue                                                                                                                                                                        | not defined             |
| `RCH_DATABASE_MAX_CONNECTIONS`      | No                          | [Bulk] Connections created for the database pool                                                                                                                                                                                            | 5                       |
| `RCH_MINIMUM_TASK_CONCURRENCY`      | No                          | [Bulk] Minimum number of concurrent running tasks below which more tasks are fetched                                                                                                                                                        | 10                      |
| `RCH_MAXIMUM_CONCURRENT_TASK_FETCH` | No                          | [Bulk] Maximum number of tasks fetched at once                                                                                                                                                                                              | 20                      |
| `RCH_HTTP_HOST`                     | No                          | The host name to bind the HTTP server to.                                                                                                                                                                                                   | `127.0.0.1`             |
| `PORT`                              | No                          | The port to bind the HTTP server to, often populated by the cloud provider.                                                                                                                                                                 | `8080`                  |
| `RCH_SENTRY_DSN`                    | No                          | If set, bug reports will be sent to this [Sentry](https://sentry.io) DSN.                                                                                                                                                                   | not defined             |
| `RCH_HEADER_SECRET`                 | No                          | If set, then all HTTP requests must have the `x-reacher-secret` header set to this value. This is used to protect the backend against public unwanted HTTP requests.                                                                        | undefined               |
| `RCH_FROM_EMAIL`                    | No                          | Email to use in the `<MAIL FROM:>` SMTP step. Can be overwritten by each API request's `from_email` field.                                                                                                                                  | reacher.email@gmail.com |
| `RCH_HELLO_NAME`                    | No                          | Name to use in the `<EHLO>` SMTP step. Can be overwritten by each API request's `hello_name` field.                                                                                                                                         | gmail.com               |
| `RCH_WEBDRIVER_ADDR`                | No                          | Set to a running WebDriver process endpoint (e.g. `http://localhost:9515`) to use a headless navigator to password recovery pages to check Yahoo and Hotmail/Outlook addresses. We recommend `chromedriver` as it allows parallel requests. | not defined             |

## REST API Documentation

Read docs on https://help.reacher.email/rest-api-documentation.

The API exposes the following endpoint: `POST /v0/check_email` expecting the following body:

```js
{
    "to_email": "someone@gmail.com",
    "from_email": "my@my-server.com", // (optional) email to use in the `FROM` SMTP command, defaults to "user@example.org"
    "hello_name": "my-server.com",    // (optional) name to use in the `EHLO` SMTP command, defaults to "localhost"
    "proxy": {                        // (optional) SOCK5 proxy to run the verification through, default is empty
        "host": "my-proxy.io",
        "port": 1080,
        "username": "me",             // (optional) Proxy username
        "password": "pass"            // (optional) Proxy password
    },
    "smtp_port": 587                  // (optional) SMTP port to do the email verification, defaults to 25
}
```

For example, you can send the following `curl` request:

```bash
curl -X POST \
    -H'Content-Type: application/json' \
    -d'{"to_email":"someone@gmail.com"}' \
    http://localhost:8080/v0/check_email
```

Also check the [OpenAPI documentation](https://help.reacher.email/rest-api-documentation).

## Build From Source

You can build the backend from source to generate a binary, and run the server locally on your machine. First, [install Rust](https://www.rust-lang.org/tools/install); you'll need Rust 1.37.0 or later. Make sure `openssl` is installed too. Then, run the following commands:

```bash
# Download the code
$ git clone https://github.com/reacherhq/check-if-email-exists
$ cd check-if-email-exists

# Build the backend binary in release mode (more performant).
$ cargo build --release --bin reacher_backend

# Run the binary with some useful logs.
$ RUST_LOG=info ./target/release/reacher_backend
```

The server will then be listening on `http://127.0.0.1:8080`.

## Prune DB

-   Start a PostgreSQL Server
-   Start Reacher with Bulk Endpoints enabled.
    -   e.g `.env` :
        RCH_ENABLE_BULK=1
        DATABASE_URL="postgresql://user:temporary@localhost"
-   Inside `.env` set `DAYS_OLD` e.g 1,2 etc
-   Send a request to the Bulk End point and wait for the job id to be allotted.
-   Build and Run `script` with `cargo run --bin prune_db`
