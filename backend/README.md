[![Actions Status](https://github.com/reacherhq/backend/workflows/pr/badge.svg)](https://github.com/reacherhq/backend/actions)
[![Github Sponsor](https://img.shields.io/static/v1?label=Sponsor&message=%E2%9D%A4&logo=GitHub&link=https://github.com/sponsors/amaurym)](https://github.com/sponsors/amaurym)

<br /><br />

<p align="center"><img align="center" src="https://storage.googleapis.com/saasify-uploads-prod/696e287ad79f0e0352bc201b36d701849f7d55e7.svg" height="96" alt="reacher" /></p>
<h1 align="center">⚙️ Reacher Backend</h1>
<h4 align="center">Backend Server for Reacher Email Verification API: https://reacher.email.</h4>

<br /><br />

This repository holds the backend for [Reacher](https://reacher.email). The backend is a HTTP server with the following components:

-   [`check-if-email-exists`](https://github.com/reacherhq/check-if-email-exists), which performs the core email verification logic,
-   [`warp`](https://github.com/seanmonstar/warp) web framework.

## ⚠️ Importance Notice: WIP branch

The `master` branch you are viewing now contains Work in Progress code on the [bulk API endpoint](https://github.com/orgs/reacherhq/projects/1). Some beta Docker images are also shipped for early usage. Please note that **the API might change** and the **code is not considered stable or production-ready**. Please use this branch only if you know what you are doing.

For the latest stable realease, please use [v0.3.12](https://github.com/reacherhq/backend/tree/v0.3.12).

## Get Started

There are 2 ways you can run this backend.

### 1. Use Docker

The [Docker image](./Dockerfile) is hosted on Docker Hub: https://hub.docker.com/r/reacherhq/backend.

To run it, run the following command:

```bash
docker run -p 8080:8080 reacherhq/backend
```

You can then send a POST request with the following body to `http://localhost:8080/v0/check_email`:

```js
{
	"to_email": "someone@gmail.com",
	"from_email": "my@my-server.com", // (optional) email to use in the `FROM` SMTP command, defaults to "user@example.org"
	"hello_name": "my-server.com",    // (optional) name to use in the `EHLO` SMTP command, defaults to "localhost"
	"proxy": {                        // (optional) SOCK5 proxy to run the verification through, default is empty
		"host": "my-proxy.io",
		"port": 1080
	},
	"smtp_port": 587                  // (optional) SMTP port to do the email verification, defaults to 25
}
```

### 2. Run locally

If you prefer to run the server locally on your machine, just clone the repository and run:

```bash
cargo run
```

The server will then be listening on `http://127.0.0.1:8080`.

### Configuration

These are the environment variables used to configure the HTTP server:

| Env Var                             | Required?                   | Description                                                                                               | Default            |
| ----------------------------------- | --------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------ |
| `RCH_ENABLE_BULK`                   | No                          | If set to 1, then bulk verification endpoints will be added to the backend.                               | 0                  |
| `DATABASE_URL`                      | Yes if `RCH_ENABLE_BULK==1` | Database connection string for storing results and task queue                                             | not defined        |
| `RCH_HTTP_HOST`                     | No                          | The host name to bind the HTTP server to.                                                                 | `127.0.0.1`        |
| `PORT`                              | No                          | The port to bind the HTTP server to, often populated by the cloud provider.                               | `8080`             |
| `RCH_FROM_EMAIL`                    | No                          | The email to use in the `MAIL FROM:` SMTP command.                                                        | `user@example.org` |
| `RCH_SENTRY_DSN`                    | No                          | If set, bug reports will be sent to this [Sentry](https://sentry.io) DSN.                                 | not defined        |
| `RCH_DATABASE_MAX_CONNECTIONS`      | No                          | Connections created for the database pool                                                                 | 5                  |
| `RCH_MINIMUM_TASK_CONCURRENCY`      | No                          | Minimum number of concurrent running tasks below which more tasks are fetched                             | 10                 |
| `RCH_MAXIMUM_CONCURRENT_TASK_FETCH` | No                          | Maximum number of tasks fetched at once                                                                   | 20                 |
| `RUST_LOG`                          | No                          | One of `trace,debug,warn,error,info`. 💡 PRO TIP: `RUST_LOG=debug` is very handful for debugging purposes. | not defined        |

## REST API Documentation

Read docs on https://help.reacher.email/rest-api-documentation.

The API basically only exposes one endpoint: `POST /v0/check_email` expecting the following body:

```js
{
	"to_email": "someone@gmail.com",
	"from_email": "my@my-server.com", // (optional) email to use in the `FROM` SMTP command, defaults to "user@example.org"
	"hello_name": "my-server.com",    // (optional) name to use in the `EHLO` SMTP command, defaults to "localhost"
	"proxy": {                        // (optional) SOCK5 proxy to run the verification through, default is empty
		"host": "my-proxy.io",
		"port": 1080
	},
	"smtp_port": 587                  // (optional) SMTP port to do the email verification, defaults to 25
}
```

Also check [`openapi.json`](./openapi.json) for the complete OpenAPI specification.

## License

`reacherhq/backend`'s source code is provided under a **dual license model**.

### Commercial license

If you want to use `reacherhq/backend` to develop commercial sites, tools, and applications, the Commercial License is the appropriate license. With this option, your source code is kept proprietary. Purchase an `reacherhq/backend` Commercial License at https://reacher.email/pricing.

### Open source license

If you are creating an open source application under a license compatible with the GNU Affero GPL license v3, you may use `reacherhq/backend` under the terms of the [AGPL-3.0](./LICENSE.AGPL).

[Read more](https://help.reacher.email/reacher-licenses) about Reacher's license.

## Sponsor my Open-Source Work

If you like my open-source work at Reacher, consider [sponsoring me](https://github.com/sponsors/amaurym/)! You'll also get 8000 free email verifications every month with your Reacher account, and a this contribution would mean A WHOLE LOT to me.
