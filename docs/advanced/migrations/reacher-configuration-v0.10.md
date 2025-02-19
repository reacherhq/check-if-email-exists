# Reacher Configuration (v0.10)

You can find below the exhaustive list of configurable parameters to optimize Reacher.

To tweak a configuration, look at the "Env variable" name in the comments, and pass in the `-e ENV_VAR=VALUE` flag to Docker. See [#examples-with-docker](reacher-configuration-v0.10.md#examples-with-docker "mention").

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

# Optional Sentry DSN. If set, all errors will be sent to Sentry.
#
# Env variable: RCH__SENTRY_DSN
# sentry_dsn = "<PASTE_YOUR_DSN_NOW>"

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

# Throttle the maximum number of requests per second, per minute, per hour, and
# per day for this worker.
# All fields are optional; comment them out to disable the limit.
#
# We however recommend setting the throttle for at least the per-minute and
# per-day limits to prevent the IPs from being blocked by the email providers.
# The default values are set to 60 requests per minute and 10,000 requests per
# day.
#
# Important: these throttle configurations only apply to /v1/* endpoints, and
# not to the previous /v0/check_email endpoint. The latter endpoint always
# executes the verification immediately, regardless of the throttle settings.
#
# Env variables:
# - RCH__THROTTLE__MAX_REQUESTS_PER_SECOND
# - RCH__THROTTLE__MAX_REQUESTS_PER_MINUTE
# - RCH__THROTTLE__MAX_REQUESTS_PER_HOUR
# - RCH__THROTTLE__MAX_REQUESTS_PER_DAY
[throttle]
# max_requests_per_second = 20
max_requests_per_minute = 60
# max_requests_per_hour = 1000
max_requests_per_day = 10000

# Configuration for a queue-based architecture for Reacher. This feature is
# currently in **beta**. The queue-based architecture allows Reacher to scale
# horizontally by running multiple workers that consume emails from a RabbitMQ
# queue.
#
# To enable the queue-based architecture, set the "enable" field to "true" and
# configure the RabbitMQ connection below. The "concurrency" field specifies
# the number of concurrent emails to verify for this worker.
#
# For more information, see the documentation at:
# https://docs.reacher.email/self-hosting/scaling-for-production
[worker]
# Enable the worker to consume emails from the RabbitMQ queues. If set, the
# RabbitMQ configuration below must be set as well.
#
# Env variable: RCH__WORKER__ENABLE
enable = false

# RabbitMQ configuration.
[worker.rabbitmq]
# Env variable: RCH__WORKER__RABBITMQ__URL
url = "amqp://guest:guest@localhost:5672"

# Number of concurrent emails to verify for this worker.
#
# Env variable: RCH__WORKER__RABBITMQ__CONCURRENCY
concurrency = 5

# Below are the configurations for the storage of the email verification
# results. We currently support the following storage backends:
# - Postgres
#
# Uncomment the following line to configure the storage to use Postgres.
# [storage.postgres]

# # URL to connect to the Postgres database.
#
# Env variable: RCH__STORAGE__POSTGRES__DB_URL
# db_url = "postgresql://localhost/reacherdb"
#
# If you wish to store additional data along with the verification results,
# you can add a JSON object to the "extra" field. This object will be stored
# as a JSONB column in the database. This is for example useful to track who
# initiated the verification request in a multi-tenant system.
# 
# Env variable: RCH__STORAGE__0__POSTGRES__TABLE_NAME
# extra = { "my_custom_key" = "my_custom_value" }
```

## Examples with Docker

To overwrite the EHLO/HELO name:

```bash
docker run -e RCH__HELLO_NAME=my.company.com -p 8080:8080 reacherhq/backend:beta
```

To store all email verification results to a Postgres database:

```bash
docker run -e RCH__STORAGE__POSTGRES__DB_URL="postgres://user:pass@mydomain.mycompany.com/my_db_name" -p 8080:8080 reacherhq/backend:beta
```

For advanced users, if you prefer to pass in the full [`backend_config.toml`](../../../backend/backend_config.toml) file instead of individual environment variable flags, run:

```bash
docker run -e RUST_LOG=reacher=debug -v /path/to/local/backend_config.toml:./backend_config.toml -p 8080:8080 reacherhq/backend:beta
```

We recommend passing in `-e RUST_LOG=reacher=debug`, at least on first run, as the debug logs will show the final configuration parsed by Reacher.
