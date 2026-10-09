# Install Reacher in 20min

Reacher is designed for seamless self-hosting, giving you full control over its operation on your infrastructure. This guide demonstrates how to install and run Reacher on your local computer in under 20 minutes, using the official [Docker image](https://hub.docker.com/r/reacherhq/backend).

## Prerequisites

* Docker installed on your system (follow the [Docker installation guide](https://docs.docker.com/get-docker/) for your OS).

## Tutorial Scope: Install Reacher on your local computer

Reacher’s stateless architecture enables easy horizontal scaling by deploying multiple containers, each running a Reacher instance for parallel email verifications. However, for simplicity, this tutorial focuses on a local installation. A further section focuses on [scaling-for-production](scaling-for-production/ "mention").

{% hint style="warning" %}
Reacher performs SMTP verifications over port 25. Many ISPs and cloud providers block outgoing requests on port 25, in which case you'll need to configure a proxy, see [proxies](proxies/ "mention").
{% endhint %}

## Step-by-Step Tutorial

1. Run Reacher's latest [Docker image](https://hub.docker.com/r/reacherhq/backend):

```bash
docker run -p 8080:8080 reacherhq/backend:latest
```

Expected output:

```bash
2024-09-19T12:58:32.918254Z  INFO reacher: Running Reacher version="0.10.0"
Starting ChromeDriver 124.0.6367.78 (a087f2dd364ddd58b9c016ef1bf563d2bc138711-refs/branch-heads/6367@{#954}) on port 9515
Only local connections are allowed.
Please see https://chromedriver.chromium.org/security-considerations for suggestions on keeping ChromeDriver safe.
ChromeDriver was started successfully.
2024-09-19T12:58:32.976589Z  INFO reacher: Server is listening host=0.0.0.0 port=80
```

If you see errors, either try [debugging-reacher.md](debugging-reacher.md "mention") or send an email to [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention").

Advanced users can also set additional [reacher-configuration-v0.10.md](reacher-configuration-v0.10.md "mention").

2. Verify an email by running the following command in another terminal.

<pre class="language-bash"><code class="lang-bash"><strong>curl -X POST \
</strong>	-H'Content-Type: application/json' \
	-d'{"to_email":"amaury@reacher.email"}' \
	http://localhost:8080/v1/check_email
</code></pre>

Advanced users can pass additional configuration fields to the  [v1-check\_email.md](../advanced/openapi/v1-check_email.md "mention") endpoint.

3. If successful, you'll see JSON object with an `is_reachable` field.

```json
{
    "input": "amaury@reacher.email",
    "is_reachable": "safe",
    // --snip--
}
```

You can read more about all the fields in [is-reachable.md](../getting-started/is-reachable.md "mention").

{% hint style="warning" %}
If this step hangs for a long time, or returns a JSON result with `is_reachable="unknown"`, see [debugging-reacher.md](debugging-reacher.md "mention")on how to fix this.
{% endhint %}

If you go back to check the terminal with the Docker command, you should see corresponding logs:

```log
// --snip--
2024-12-15T11:33:36.169891Z  INFO reacher: Starting verification email="amaury@reacher.email"
2024-12-15T11:33:45.015130Z  INFO reacher: Done verification email="amaury@reacher.email" is_reachable=Safe
```

Congratulations! You just successfully verified an email from your computer. Now it's time to think about [scaling-for-production](scaling-for-production/ "mention").

## Troubleshooting

If you have any issue in one of the steps above, you can try [debugging-reacher.md](debugging-reacher.md "mention")yourself, or send me an email [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention").
