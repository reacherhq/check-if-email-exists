# Reacher Configuration (v0.10)

{% hint style="info" %}
This configuration has been introduced in v0.10, which is still in `beta`. For the stable 0.7 version, please see [docker-environment-variables.md](../../self-hosting/docker-environment-variables.md "mention").
{% endhint %}

Previously, in v0.7, configuration was done solely via environment variables. Given the growing amount of configurable parameters, we now offer a file-based configuration too, on top of environment variables.

You can find below the exhaustive list of configurations, as well as their corresponding environment variable.

```toml
# Backend configuration.

# Name to identify the backend.
#
# Env variable: RCH__BACKEND_NAME
backend_name = "backend-dev"

# Host to bind the backend to.
#
# Env variable: RCH__HTTP_HOST
http_host = "127.0.0.1"

# Port for the backend.
#
# Env variable: RCH__HTTP_PORT
http_port = 8080

# Shared secret between a trusted client and the backend, required in the
# `x-reacher-secret` header of all incoming requests.
#
# Env variable: RCH__HEADER_SECRET
# header_secret = "my-secret"

# Name to use during the EHLO/HELO command in the SMTP conversation.
# Ideally, this should match the reverse DNS of the server's IP address.
#
# Env variable: RCH__HELLO_NAME
hello_name = "localhost"

# Email to use during the MAIL FROM command in the SMTP conversation.
# Ideally, the domain of this email should match the "hello_name" above.
#
# Env variable: RCH__FROM_EMAIL
from_email = "hello@localhost"

# Address of the Chrome WebDriver server for headless email verifications.
#
# Env variable: RCH__WEBDRIVER_ADDR
webdriver_addr = "http://localhost:9515"

# Timeout for each SMTP connection, in seconds. Leaving it commented out will
# not set a timeout, i.e. the connection will wait indefinitely.
#
# Env variable: RCH__SMTP_TIMEOUT
# smtp_timeout = 45

# Uncomment the lines below to route all SMTP verification requests
# through a specified proxy. Note that the proxy must be a SOCKS5 proxy to work
# with the SMTP protocol. This proxy will not be used for headless
# verifications.
#
# The username and password are optional and only needed if the proxy requires
# authentication.
#
# Env variables:
# - RCH__PROXY__HOST
# - RCH__PROXY__PORT
# - RCH__PROXY__USERNAME
# - RCH__PROXY__PASSWORD
#
# [proxy]
# host = "my.proxy.com"
# port = 1080
# username = "my-username"
# password = "my-password"

# Verification method to use for each email provider. Available methods are:
# "smtp", "headless", and "api". Note that not all methods are supported by
# all email providers.
[verif_method]
# Gmail currently only supports the "smtp" method.
#
# Env variable: RCH__VERIF_METHOD__GMAIL
gmail = "smtp"
# Hotmail B2B currently only supports the "smtp" method.
#
# Env variable: RCH__VERIF_METHOD__HOTMAILB2B
hotmailb2b = "smtp"
# Hotmail B2C supports both "headless" and "smtp" methods. The "headless"
# method is recommended.
hotmailb2c = "headless"
# Yahoo supports both "headless" and "smtp" methods. The "headless" method is
# recommended.
yahoo = "headless"

[worker]
# Enable the worker to consume emails from the RabbitMQ queues. If set, the
# RabbitMQ configuration below must be set as well.
#
# Env variable: RCH__WORKER__ENABLE
enable = true

# RabbitMQ configuration.
[worker.rabbitmq]
# Env variable: RCH__WORKER__RABBITMQ__URL
url = "amqp://guest:guest@localhost:5672"

# Queues to consume emails from. By default, the worker consumes from all
# queues.
#
# To consume from only a subset of queues, uncomment the line `queues = "all"`
# and specify the queues you want to consume from.
#
# Below is the exhaustive list of queue names that the worker can consume from:
# - "check.gmail": subscribe exclusively to Gmail emails.
# - "check.hotmailb2b": subscribe exclusively to Hotmail B2B emails.
# - "check.hotmailb2c": subscribe exclusively to Hotmail B2C emails.
# - "check.yahoo": subscribe exclusively to Yahoo emails.
# - "check.everything_else": subscribe to all emails that are not Gmail, Yahoo, or Hotmail.
#
# Env variable: RCH__WORKER__RABBITMQ__QUEUES
#
# queues = ["check.gmail", "check.hotmail.b2b", "check.hotmail.b2c", "check.yahoo", "check.everything_else"]
queues = "all"

# Number of concurrent emails to verify for this worker across all queues.
#
# Env variable: RCH__WORKER__RABBITMQ__CONCURRENCY
concurrency = 20

# Throttle the maximum number of requests per second, per minute, per hour, and
# per day for this worker.
# All fields are optional; comment them out to disable the limit.
#
# Important: these throttle configurations only apply to /v1/* endpoints, and
# not to the previous /v0/check_email endpoint. The latter endpoint always
# executes the verification immediately, regardless of the throttle settings.
#
# Env variables:
# - RCH__WORKER__THROTTLE__MAX_REQUESTS_PER_SECOND
# - RCH__WORKER__THROTTLE__MAX_REQUESTS_PER_MINUTE
# - RCH__WORKER__THROTTLE__MAX_REQUESTS_PER_HOUR
# - RCH__WORKER__THROTTLE__MAX_REQUESTS_PER_DAY
[worker.throttle]
# max_requests_per_second = 20
# max_requests_per_minute = 100
# max_requests_per_hour = 1000
# max_requests_per_day = 20000

# Postgres configuration. Currently, a Postgres database is required to store
# the results of the verifications. This might change in the future, allowing
# for pluggable storage.
[worker.postgres]
# Env variable: RCH__WORKER__POSTGRES__DB_URL
db_url = "postgresql://localhost/reacherdb"

# Optional Sentry DSN. If set, all errors will be sent to Sentry.
#
# Env variable: RCH__SENTRY_DSN
# sentry_dsn = "<PASTE_YOUR_DSN_NOW>"
```

## Usage with Docker

You can continue using environment variables with Docker. For example, to overwrite the EHLO/HELO name, simply run:

```bash
docker run -e RCH__HELLO_NAME=my.company.com -p 8080:8080 reacherhq/backend:beta
```

However, if you prefer to pass in a local `backend_config.toml` file instead, run:

```bash
docker run -e RUST_LOG=reacher=debug -v /path/to/local/backend_config.toml:./backend_config.toml -p 8080:8080 reacherhq/backend:beta
```

We recommend passing in `-e RUST_LOG=reacher=debug`, at least on first run, as the debug logs will show the final configuration parsed by Reacher.
