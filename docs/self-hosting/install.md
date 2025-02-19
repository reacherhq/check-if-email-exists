# Install Reacher in 20min

Reacher is designed for seamless self-hosting, giving you full control over its operation on your infrastructure. This guide demonstrates how to install and run Reacher on your local computer in under 20 minutes, using a Dockerfile that is provided you as part of the [commercial-license-trial.md](licensing/commercial-license-trial.md "mention").

## Prerequisites

* An account on [https://reacher.email](https://reacher.email) (required for the Commercial License Trial and access to the Dockerfile).
* Docker installed on your system (follow the [Docker installation guide](https://docs.docker.com/get-docker/) for your OS).

## Tutorial Scope: Install Reacher on your local computer

Reacher’s stateless architecture enables easy horizontal scaling by deploying multiple containers, each running a Reacher instance for parallel email verifications. However, for simplicity, this tutorial focuses on a local installation. A further section focuses on [scaling-for-production](scaling-for-production/ "mention").

The provided Dockerfile includes a pre-configured proxy, resolving the common ISP restriction on outgoing requests to port 25 used by Reacher to perform SMTP verifications.

<details>

<summary>Understand the features and limitations of the Commercial License Trial.</summary>

The Dockerfile provided as part of the Commercial License Trial is designed to enable quick setup for email verifications. Below are its key features and limitations:

* **Built-in Proxy Configuration**: we use [**Proxy4Smtp**](https://www.proxy4smtp.com), a 3rd-party proxy with carefully maintained IPs optimized for SMTP verifications. This ensures reliable email verification even in cloud environments with restricted SMTP access. Learn more in [proxies](proxies/ "mention").
* **Daily Verification Limit**: capped at 60 per minute at **10,000 per day**.
* **Usage Tracking**: verification results are anonymized and sent back to Reacher, and used to monitor daily usage and detect potential abuse.

You can also read more in [commercial-license-trial.md](licensing/commercial-license-trial.md "mention").

</details>

## Step-by-Step Tutorial

1. Navigate to the **Commercial License Trial** tab of your Reacher Dashboard ([go there directly](https://app.reacher.email/en/dashboard/commercial_license)). You'll see a command to run Reacher's latest (v0.10) [Docker image](https://hub.docker.com/r/reacherhq/backend):

```bash
docker run -e RCH__COMMERCIAL_LICENSE_TRIAL__API_TOKEN=<YOUR_UNIQUE_TOKEN> -p 8080:8080 reacherhq/commercial-license-trial:latest # v0.10
```

Replace `<YOUR_UNIQUE_TOKEN>` with your unique API token shown in the dashboard.

Expected output:

```bash
2024-09-19T12:58:32.918254Z  INFO reacher: Running Reacher version="0.10.0"
Starting ChromeDriver 124.0.6367.78 (a087f2dd364ddd58b9c016ef1bf563d2bc138711-refs/branch-heads/6367@{#954}) on port 9515
Only local connections are allowed.
Please see https://chromedriver.chromium.org/security-considerations for suggestions on keeping ChromeDriver safe.
ChromeDriver was started successfully.
2024-09-19T12:58:32.976589Z  INFO reacher: Server is listening host=0.0.0.0 port=80
```

If you see an error message, such as `` Error: missing field `api_token` ``, double-check the `-e RCH__COMMERCIAL_LICENSE_TRIAL__API_TOKEN` flag you passed. If you see other errors, either try [debugging-reacher.md](debugging-reacher.md "mention") or send an email to [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention").

Advanced users can also set additional [reacher-configuration-v0.10.md](reacher-configuration-v0.10.md "mention").

4. Verify an email by running the following command in another terminal.

<pre class="language-bash"><code class="lang-bash"><strong>curl -X POST \
</strong>	-H'Content-Type: application/json' \
	-d'{"to_email":"amaury@reacher.email"}' \
	http://localhost:8080/v1/check_email
</code></pre>

Advanced users can pass additional configuration fields to the  [v1-check\_email.md](../advanced/openapi/v1-check_email.md "mention") endpoint.

4. If successful, you'll see JSON object with an `is_reachable` field.

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
