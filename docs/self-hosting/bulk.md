# Bulk Verification

The default Reacher API only exposes one endpoint, `/v0/check_email`, which allows verifying one email at a time. The optional Bulk Verification API allows you to queue up a list of emails in one go.

## Prerequisites

* A self-hosted setup, see [install.md](install.md "mention").
* A PostgreSQL database, you can start for free with [Supabase](https://supabase.com/) (no affiliation).

## Get Started

When running the Reacher backend, set two new environment variables:

* &#x20;`RCH_ENABLE_BULK=1`
* &#x20;`DATABASE_URL=<your_postgres_db>`

For example, if running with docker, run

```bash
docker run \
	-e RCH_ENABLE_BULK=1 \
	-e DATABASE_URL=<your_postgres_db> \
	# Other flags
	reacherhq/backend:v0.7.0
```

The `DATABASE_URL` value should look like `postgres://<user>:<password>@<hostname>/<db_name>`.You should see the backend running with the following logs:

```bash
[2022-08-15T19:54:23Z INFO  reacher] Running Reacher v0.7.0
[2022-08-15T19:54:23Z INFO  reacher] Bulk endpoints enabled.
Server is listening on 0.0.0.0:8080.
```

## How does Bulk email verification work?

&#x20;

<figure><img src="https://www.notion.so/image/https%3A%2F%2Ffile.notion.so%2Ff%2Ff%2F8fc90893-c1e2-426b-8b22-6e4e323244db%2F3fff2c3e-e65f-4306-abbd-b4037760b517%2Fbulk.svg%3Ftable%3Dblock%26id%3D2090bf73-99bb-462b-90f2-57b0af6733ab%26spaceId%3D8fc90893-c1e2-426b-8b22-6e4e323244db%26expirationTimestamp%3D1726833600000%26signature%3DCMRYdgntz2SsZnL8Krlve4pJS6mUB666K64SV9CiTHM?table=block&#x26;id=2090bf73-99bb-462b-90f2-57b0af6733ab&#x26;cache=v2" alt="Flow chart describing the bulk verification process."><figcaption><p>Flow chart describing the bulk verification process.</p></figcaption></figure>

Bulk email verification is done in 3 steps:

#### **1. Submit a list of emails: `POST /v0/bulk`**

The body of the request contains the list of emails, as well as a couple of configuration options.

```json
{
    // Required fields:
    "input_type": "array",             // Must be "array". Future versions might allow CSV uploading.
    "input": [                         // Endpoint accepts a list of emails.
        "support@reacher.email",
        "invalid@reacher.email"
    ],

    // All fields below are optional:
    "proxy": {
        "host": "my.proxy.com",
        "port": "9080",
        "username": "user",           // Optional authentication for proxy.
        "password": "pass",
    },
    "hello_name": "my.domain.com",    // The value to use in the EHLO handshake.
    "from_email": "me@my.domain.com", // The value to use in the MAIL FROM command.
    "smtp_ports": [25, 587]           // List of SMT ports to try for each email, in given order. Defaults to [25].
}
```

If successful, this endpoint will return a unique job ID, used to track the progress of the bulk verification job and fetch its results.

```json
{
    "job_id": 150970
}
```

#### **2. Verify the status of the job: `GET /v0/bulk/{job_id}`**

If the list of emails is long, then the bulk verification job can take some time. Ping regularly on the endpoint above to see the status of the job.When the job is still running, the `job_status` will be `Running`:

```json
{
    "job_id": 150970,                            // From previous step.
    "created_at": "2017-04-15T20:00:06:00.000Z", // Start time of the job.
    "finished_at": null,                         // Stays `null` as long as job is still running.
    "total_records": 24606,
    "total_processed": 10,                       // Shows job progress.
    "summary": {                                 // Summary of the list's health.
        "total_safe": 5,
        "total_invalid": 2,
        "total_risky": 2,
        "total_unknown": 1
    },
    "job_status": "Running"                      // Wait for "Completed".
}
```

And when the job is finished, we get `job_status = Completed`, and the `finished_at` field will be populated with the job’s end time.

#### **3. Download the job results: `GET /v0/bulk/{job_id}/results`**

Once the `job_status` field in from the previous step is `Completed`, this endpoint will show the results of all the emails in the list.

```json
{
    "results": [
		  {
          "input": "someone@gmail.com",
          "is_reachable": "risky",
          // --snip: all fields--
      },
      // --snip: other results--
    ]
}
```

To avoid returning a huge JSON payload, the `results` array by default only returns the first 50 email results. We recommend using pagination on the client side, using the 2 following query parameters:

* `?offset=<n>`: The offset from which we return the results, which is equivalent to the number of elements in the array to skip. Defaults to `0`.
* `?limit=<n>`: The number of results to return. Defaults to `50`.

For example, if your initial input has 100 emails to verify, and you want the results for emails #61-#70, you should add the query parameters: `GET /v0/bulk/{job_id}/results?offset=60&limit=10`.&#x20;

{% hint style="success" %}
Pro Tip: You can also download the results as CSV, by passing the `?format=csv` query paramter: `GET /v0/bulk/{job_id}/results?format=csv`.
{% endhint %}

## Questions?

This Bulk email verification feature is still new, so feel free to send me an email [amaury](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention").
