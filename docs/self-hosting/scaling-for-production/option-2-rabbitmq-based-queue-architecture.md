# Option 2: RabbitMQ-based Queue Architecture

Reacher includes an optional, opinionated queue-based architecture designed to scale efficiently and handle high email verification volumes. This architecture comprises 4 main components and is highly configurable to meet specific business needs.

<table><thead><tr><th width="183">Component</th><th width="295">Description</th><th>Docker image</th></tr></thead><tbody><tr><td><strong>HTTP server</strong></td><td>Accepts incoming email verification requests, and post them into the queue.</td><td><code>reacherhq/commercial-license-trial</code></td></tr><tr><td><a href="https://rabbitmq.com">RabbitMQ</a></td><td>Reacher uses a reliable, mature and open-source queue implementation.</td><td><code>rabbitmq:4.0-management</code></td></tr><tr><td><strong>Workers</strong></td><td>One or more consumers of the queue, which perform the actual email verification task.</td><td><code>reacherhq/commercial-license-trial</code></td></tr><tr><td><strong>Storage</strong></td><td>A place to store all results, currently only PostgresDB is supported.</td><td><code>postgres:14</code></td></tr></tbody></table>

Note that Reacher provides the same Docker image `reacherhq/commercial-license-trial` which can act as both a **Worker** and a **HTTP server**.

<figure><img src="../../.gitbook/assets/Screenshot 2024-11-30 at 15.33.27.png" alt=""><figcaption><p>Reacher queue architecture</p></figcaption></figure>

With this architecture, it's possible to horizontally scale the number of workers. However, to prevent spawning too many workers at once resulting in blacklisted IPs, we need to configure some concurrency and throttling parameters below.

### Worker Configuration

#### **Enabling the Architecture**

To enable the worker-based architecture, configure the following parameters in your deployment:

* `worker.enable`: Set to `true` to activate the worker role.
* `worker.rabbitmq.url`: URL of the RabbitMQ instance for task queuing.
* `postgres.db_url`: URL of a PostgreSQL database to store verification results.

#### Using Proxies for Workers

Since spawning workers on cloud providers doesn't guarantee a reputable IP assigned to the worker, we configure all workers to use a proxy.

* `proxy.{host,port}`: Set a proxy to route all SMTP requests through. You can optionally pass in `username` and `password` if required.

{% hint style="info" %}
The Dockerfile provided in the Commercial License Trial already has these parameters set up.
{% endhint %}

#### Concurrency and Throttling Parameters

To prevent IP blacklisting, configure the following parameters:

* **Concurrency**:
  * `worker.rabbitmq.concurrency`: Set to `5`. Each worker processes up to 5 emails concurrently, which means each proxy IP address is perform maximum 5 email verifications at any give time.
* **Throttling**:
  * `throttle.max_requests_per_minute`: Set to `60`. Limits request spikes to prevent SMTP server flags.
  * `throttle.max_requests_per_day`: Set to `10,000`. Aligns with the recommended verifications per proxy IP.

{% hint style="info" %}
The Dockerfile provided in the Commercial License Trial already has these parameters set up.
{% endhint %}

## Understanding the architecture with Docker Compose

We do not recommend using Docker Compose for a high-volume production setup. However, for understanding or learning the architecture, this [`docker_compose.yaml`](../../../docker-compose.yaml) file can be useful.

## More questions?

Contact [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention")if you have more questions about this architecture, such as:

* deploying on Kubernetes (Ansible playbook, Pulumi)
* more specialized workers (e.g. some workers doing headless verification only, others doing SMTP only)
