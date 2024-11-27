# Install Reacher in 20min

Reacher is built with self-hosting as a primary feature, giving you full control over how the service operates on your infrastructure. It can be deployed on a single server in under 20 minutes. This tutorial shows you how.

{% hint style="info" %}
You can run this tutorial without a Commercial License, as a Free Trial. Read more about the Free Trial in [licensing.md](licensing.md "mention").
{% endhint %}

## Tutorial Scope: Install Reacher on a single server

Reacher is **stateless** by design, meaning you can deploy multiple containers, each running a separate instance of Reacher, to perform email verifications in parallel. This architecture enables easy horizontal scaling.

However, for the sake of this tutorial, we will install Reacher on a single dedicated server. This allows minimal setup to get Reacher working, and ensures that the chosen cloud provider allows outgoing port 25 requests.

If you're interested in ideas for a production deployment setup, skip to [scaling-for-production.md](scaling-for-production.md "mention").

## Step-by-Step Tutorial

1. Install Docker on your server. You can follow [Docker's guide](https://docs.docker.com/engine/install/) for your OS.
2. Run Reacher's latest (v0.7) [Docker image](https://hub.docker.com/r/reacherhq/backend):

```bash
docker run -p 8080:8080 reacherhq/backend:latest # v0.7
```

You should see the following output:

```bash
2024-09-19T12:58:32.918254Z  INFO reacher: Running Reacher version="0.10.0"
Starting ChromeDriver 124.0.6367.78 (a087f2dd364ddd58b9c016ef1bf563d2bc138711-refs/branch-heads/6367@{#954}) on port 9515
Only local connections are allowed.
Please see https://chromedriver.chromium.org/security-considerations for suggestions on keeping ChromeDriver safe.
ChromeDriver was started successfully.
2024-09-19T12:58:32.976589Z  INFO reacher: Server is listening host=0.0.0.0 port=80
```

Advanced users can set additional [docker-environment-variables.md](docker-environment-variables.md "mention").

3. Make sure that you can verify an email remotely by running the following command from your local machine.

```bash
curl -X POST \
	-H'Content-Type: application/json' \
	-d'{"to_email":"amaury@reacher.email"}' \
	http://<IP_OF_YOUR_SERVER>:8080/v0/check_email
```

{% hint style="warning" %}
If this step hangs for a long time, or returns a JSON result with `is_reachable="unknown"`, it generally means that port 25 is restricted. See [debugging-reacher.md](debugging-reacher.md "mention")on how to fix this.
{% endhint %}

4. If you see a JSON output with an `is_reachable` field, then you're set, congratulations! :tada:

## Troubleshooting

If you have any issue in one of the steps above, you can try [debugging-reacher.md](debugging-reacher.md "mention")yourself, or send me an email [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention").
