# Scaling for Production

{% hint style="info" %}
The architecture detalied below is currently only available on the v0.10 `beta` Docker version.
{% endhint %}

Reacher's stateless design allows for efficient horizontal scaling. We propose here a queue-based architecture to handle more than 10 millions of email verifications per month.

The architecture contains 4 components:

<table><thead><tr><th width="183">Component</th><th width="295">Description</th><th>Docker image</th></tr></thead><tbody><tr><td><strong>HTTP server</strong></td><td>Receives incoming email verification requests, and post them into the queue.</td><td><code>reacherhq/backend:beta</code></td></tr><tr><td><a href="https://rabbitmq.com">RabbitMQ</a></td><td>Reacher uses a reliable, mature and open-source queue implementation.</td><td><code>rabbitmq:4.0-management</code></td></tr><tr><td><strong>Workers</strong></td><td>One or more consumers of the queue, which perform the actual email verification task.</td><td><code>reacherhq/backend:beta</code></td></tr><tr><td><strong>Storage</strong></td><td>A place to store all results, currently only PostgresDB is supported.</td><td><code>postgres:14</code></td></tr></tbody></table>

Note that Reacher provides the same Docker image `reacherhq/backend` which can act as both a **Worker** and a **HTTP server**.

<figure><img src="../.gitbook/assets/Screenshot 2024-11-27 at 14.43.50.png" alt=""><figcaption><p>Reacher architecture for scaling</p></figcaption></figure>

With this architecture, it's possible to horizontally scale the number of workers, while making sure that the individual IPs don't get blacklisted. To do so, we propose to start with two types of workers.

### Common Configuration to both workers

To enable the above worker architecture, set the following parameters in [reacher-configuration-v0.10.md](../advanced/migrations/reacher-configuration-v0.10.md "mention"):&#x20;

* `worker.enable`: true
* `worker.rabbitmq.url`: Points to the URL of the RabbitMQ instance.
* `worker.postgres.db_url`: A Postgres database to store the email verification results.

### 1st worker type: SMTP worker using Proxy

These workers will consume all emails that should be verified through SMTP. Currently, this includes all emails, except Hotmail B2C and Yahoo emails, which are best verified using a headless navigator. Since maintaing IP addresses is hard, we recommend using a proxy, see [proxies.md](proxies.md "mention").

Assuming your proxy has `N` available IP addresses, we recommend spawning the same number `N` of workers, each with the config below:

* `worker.rabbitmq.queues`: `["check.gmail","check.hotmailb2b","everything_else"]`. The SMTP workers will listen to these queues.
* `worker.proxy.{host,port}`: Set a proxy to route all SMTP requests through. You can optionally pass in `username` and `password` if required.
* `worker.rabbitmq.concurrency`: 10.
* `worker.throttle.max_requests_per_minute`: 100.
* `worker.throttle.max_requests_per_day`: 10000. This is the recommended number of verifications per IP per day. Assuming there are `N` IP addresses and `N` workers, each worker should perform 10000 verifications per day.

You can scale up the number `N` as much as you need. Remember, the rule of thumb is 10000 verifications per IP per day. For example, if you're aiming for 10 millions verifications per month, we recommend 33 or 34 IPs.

```
10,000,000 emails per month / 30 = 33,000 emails per day / 10000 = 33 IPs
```

Refer to [reacher-configuration-v0.10.md](../advanced/migrations/reacher-configuration-v0.10.md "mention")to see how to set these settings.

### 2nd worker type: Headless worker

These workers will consume all emails that are best verified using a headless browser. The idea behind this verification method is to spawn a headless browser that will navigate to the email provider's password recovery page, and parse the website's response to inputting emails. This method currently works well for Hotmail and Yahoo emails.

To spawn such a worker, provide the config:

* `worker.rabbitmq.queues`: `["check.hotmailb2c","check.yahoo"]`. These are the emails that are best verified using headless.
* `worker.throttle.max_requests_per_minute`: 100

Refer to [reacher-configuration-v0.10.md](../advanced/migrations/reacher-configuration-v0.10.md "mention")to see how to set these settings.

## Understanding the architecture with Docker Compose

We do not recommend using Docker Compose for a high-volume production setup. However, for understanding the architecture, the different Docker images, as well as how to configure the workers, this [`docker_compose.yaml`](../../docker-compose.yaml) file can be useful.

## More questions?

Contact [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention")if you have more questions about this architecture, such as:

* deploying on Kubernetes (Ansible playbook, Pulumi)
* more specialized workers (e.g. Gmail and Hotmail B2B workers can be separated)
