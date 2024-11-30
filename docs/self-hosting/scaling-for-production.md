# Scaling for Production

{% hint style="info" %}
The architecture detalied below is currently only available on the v0.10 `beta` Docker version.
{% endhint %}

Reacher's stateless design allows for efficient horizontal scaling. We propose here a queue-based architecture to handle more than 10 millions of email verifications per month.

The architecture contains 4 components:

<table><thead><tr><th width="183">Component</th><th width="295">Description</th><th>Docker image</th></tr></thead><tbody><tr><td><strong>HTTP server</strong></td><td>Receives incoming email verification requests, and post them into the queue.</td><td><code>reacherhq/backend:beta</code></td></tr><tr><td><a href="https://rabbitmq.com">RabbitMQ</a></td><td>Reacher uses a reliable, mature and open-source queue implementation.</td><td><code>rabbitmq:4.0-management</code></td></tr><tr><td><strong>Workers</strong></td><td>One or more consumers of the queue, which perform the actual email verification task.</td><td><code>reacherhq/backend:beta</code></td></tr><tr><td><strong>Storage</strong></td><td>A place to store all results, currently only PostgresDB is supported.</td><td><code>postgres:14</code></td></tr></tbody></table>

Note that Reacher provides the same Docker image `reacherhq/backend` which can act as both a **Worker** and a **HTTP server**.

<figure><img src="../.gitbook/assets/Screenshot 2024-11-30 at 15.33.27.png" alt=""><figcaption><p>Reacher queue architecture</p></figcaption></figure>

With this architecture, it's possible to horizontally scale the number of workers. However, to prevent spawning to many workers at once resulting in blacklisted IPs, we need to configure some concurrency and throttling parameters below.

### Worker Configuration

To enable the above worker architecture without getting blacklisted, we need to set some parameters in [reacher-configuration-v0.10.md](../advanced/migrations/reacher-configuration-v0.10.md "mention"):

* `worker.enable`: true
* `worker.rabbitmq.url`: Points to the URL of the RabbitMQ instance.
* `worker.postgres.db_url`: A Postgres database to store the email verification results.

Since spawning workers (generally on cloud providers) doesn't guarantee a reputable IP assigned to the worker, we propose to configure all workers to use a proxy. Proxies generally offer a pricing per IP per month; we recommend buying one IP for each 10000 email verifications you do per day.

* `worker.proxy.{host,port}`: Set a proxy to route all SMTP requests through. You can optionally pass in `username` and `password` if required.

We also propose some recommended values for concurrency and throttling parameters. These parameters ensure that the proxy that we use will have its IP well maintained.

* `worker.rabbitmq.concurrency`: 5. Each worker can process 5 emails at a time.
* `worker.throttle.max_requests_per_minute`: 60. If this value is too high, the recipient SMTP server might see sudden spikes of email verifications, resulting in an IP blacklist.
* `worker.throttle.max_requests_per_day`: 10000. This is the recommended number of verifications per IP per day. Assuming our proxy has `N` IP addresses and `N` workers, each worker will perform 10000 verifications per day in average.

You can scale up the number `N` as much as you need, by buying more IPs and spawning more workers. Remember, the rule of thumb is 10000 verifications per IP per day. For example, if you're aiming for 10 millions verifications per month, we recommend buying 33 or 34 IPs:

```
10,000,000 emails per month / 30 = 33,000 emails per day / 10000 = 33 IPs
```

Refer to [reacher-configuration-v0.10.md](../advanced/migrations/reacher-configuration-v0.10.md "mention")to see how to set these settings.

## Understanding the architecture with Docker Compose

We do not recommend using Docker Compose for a high-volume production setup. However, for understanding the architecture, the different Docker images, as well as how to configure the workers, this [`docker_compose.yaml`](../../docker-compose.yaml) file can be useful.

## More questions?

Contact [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention")if you have more questions about this architecture, such as:

* deploying on Kubernetes (Ansible playbook, Pulumi)
* more specialized workers (e.g. some workers doing headless verification only, others doing SMTP only)
