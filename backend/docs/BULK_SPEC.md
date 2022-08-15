# Reacher Bulk Email Verification Specification

Author: @amaurym
Status: Implementation
Start Date: 26.12.2021
Last Updated: 10.08.2022

## Abstract

This document describes implementing the Bulk Email Verification feature on Reacher, as initially outlined in [reacherhq/backend#149](https://github.com/reacherhq/backend/issues/149).

## Pipeline

The idea is put email lists in a message queue.

![pipeline](https://files.readme.io/2ac4d24-verification_pipeline.png)

## API Endpoints Specification

This section lists all the API endpoints to implement for Reacher's bulk email verification feature.

Inspiration:

-   https://developers.neverbounce.com/docs/verifying-a-list
-   https://documentation.mailgun.com/en/latest/api-email-validation.html#bulk-validation
-   https://emailverification.whoisxmlapi.com/bulk-api/documentation/get-results-invalid-and-failed-emails

### Authentication

Authentication is not part of this specification. The repository [reacherhq/backend](https://github.com/reacherhq/backend) is auth-free. Authentication will be handled by a layer on top of reacherhq/backend, namely in [reacherhq/webapp](https://github.com/reacherhq/webapp), along with user management and billing.

### 1. `POST /v0/bulk`

Create a new job for verifying an email list. This endpoint can receive input in multiple ways. The `input_type` parameter describes the contents of the `input` parameter. `input_type` can be one of the following values:

-   `"array"`

#### Email Array

Supplying the data directly gives you the option to dynamically create email lists on the fly rather than having to write to a file. `input` will accept an array of strings, each item representing an email to verify:

Request:

```json=
{
    "input_type": "array",
    "input": [
        "support@reacher.email",
        "invalid@reacher.email"
    ],
    // All fields below are optional
    "proxy": {
        "host": "my.proxy.com",
        "port": "9080"
    },
    "hello_name": "my.domain.com",
    "from_email": "me@my.domain.com",
    "smtp_ports": [25, 587] // if error, will retry with the next port in the same task
}
```

Response:

```json=
{
    "job_id": 150970
}
```

### 2. `GET /v0/bulk/<job_id>`

Get the status of a job, as well current progress and a summary.

Response:

```json=
    "job_id": 150970,                            // auto-increment
    "created_at": "2017-04-15T20:00:06:00.000Z", // ISO 8601
    "finished_at": "2017-04-15T21:52:46:00.000Z",// undefined if job_status != "complete"
    "total_records": 24606,
    "total_processed": 24606,
    "summary": {
        "total_safe": 18227,
        "total_invalid": 1305,
        "total_risky": 4342,
        "total_unknown": 716
    },
    "job_status": "complete"                     // values: ["running", "complete"]
```

### 3. `GET /v0/bulk/<job_id>/download`

This endpoint gets the results of a job as an array of JSON. It returns an error if the job status is not `complete`.

Response:

```json=
{
    "results": [
        // Contains the exact same amount of responses as the number
        // of emails in the input.
        {
          "input": "someone@gmail.com",
          "is_reachable": "invalid",
          "misc": {
            "is_disposable": false,
            "is_role_account": false
          },
          "mx": {
            "accepts_mail": true,
            "records": [
              "alt1.gmail-smtp-in.l.google.com.",
              "alt2.gmail-smtp-in.l.google.com.",
              "alt4.gmail-smtp-in.l.google.com.",
              "alt3.gmail-smtp-in.l.google.com.",
              "gmail-smtp-in.l.google.com."
            ]
          },
          "smtp": {
            "can_connect_smtp": true,
            "has_full_inbox": false,
            "is_catch_all": false,
            "is_deliverable": false,
            "is_disabled": true
          },
          "syntax": {
            "address": "someone@gmail.com",
            "domain": "gmail.com",
            "is_valid_syntax": true,
            "username": "someone"
          }
        },
        {
          "input": "amaury@reacher.email",
          "is_reachable": "safe",
          "misc": {
            "is_disposable": false,
            "is_role_account": false
          },
          "mx": {
            "accepts_mail": true,
            "records": [
              "mail.protonmail.ch.",
              "mailsec.protonmail.ch."
            ]
          },
          "smtp": {
            "can_connect_smtp": true,
            "has_full_inbox": false,
            "is_catch_all": false,
            "is_deliverable": true,
            "is_disabled": false
          },
          "syntax": {
            "address": "amaury@reacher.email",
            "domain": "reacher.email",
            "is_valid_syntax": true,
            "username": "amaury"
          }
        },
        // more results...
    ]
}

```

### 4. Errors

In case of errors on any endpoint, the JSON to be returned is as follows.

Response:

```json=
{
    "error": "..."   // Error string
}
```

The response should also contain the correct HTTP status code.

Also see the Rust [`ReacherResponseError`](https://github.com/reacherhq/backend/blob/3f737a4c86757bc09a381c19ac4e3e759c541f54/src/errors.rs#L25) struct.

## Technical Implementation

The implementation MUST be done on the [reacherhq/backend](https://github.com/reacherhq/backend) Github repository, as a Pull Request that will be reviewed by @amaurym.

### Message Queue

The implementation of the bulk verification feature uses message queues. The queue consists of 2 processes:

-   the `web` process listens to incoming HTTP requests from end users. The `web` process already exists in the current reacherhq/backend codebase, as the [`heroku.rs`](https://github.com/reacherhq/backend/blob/master/src/bin/heroku.rs) binary. The API endpoints described in [API Endpoints Specification](#api-endpoints-specification) should be added to enable creating jobs.
-   the `worker` process does the actual email verification. It's entry point should be the binary `src/bin/worker.rs`, and will listen to jobs created by the `web` process.

We decide to use PostgreSQL as the backend for the message queue. Recent PostgreSQL versions make implementing queues much easier:

-   `SKIP LOCKED`
-   `LISTEN`/`NOTIFY`

We decide to go with the https://github.com/Diggsey/sqlxmq library. Some more advantages of PostgreSQL are outlined in the README.md

Email verification results will also be stored in the SQL database, so that users can revisit the results a long time after the job finishes.

TODO

## Q&A

This section describes Quality and Assurance of the bulk email verification feature.

### Create a successful job with remote_url

Request:

```json=
POST /v0/bulk
{
    "input_type": "remote_url",
    "input": "https://gist.githubusercontent.com/amaurym/e6a4109a2b2c9baa42806f6953b12fb3/raw/261bf800f052d63bb98d0973486f61625d316db7/reacher_bulk_qa.csv"
}
```

Expected Response:

```json=
{
    "job_id": 1
}
```

### Query job status

Request:

```json=
GET /v0/bulk/1
```

Expected Response:

When job is still running:

```json=
{
    "job_id": 150970,
    "created_at": "2017-04-15T20:00:06:00.000Z",
    "finished_at": undefined,
    "total_records": 3,
    "total_processed": 2,
    "summary": {
        "total_safe": 0,
        "total_invalid": 1,
        "total_risky": 1,
        "total_unknown": 0
    },
    "job_status": "running"
}
```

When job is complete:

```json=
{
    "job_id": 150970,
    "created_at": "2017-04-15T20:00:06:00.000Z",
    "finished_at": "2017-04-15T20:00:06:00.000Z",
    "total_records": 3,
    "total_processed": 3,
    "summary": {
        "total_safe": 0,
        "total_invalid": 1,
        "total_risky": 2,
        "total_unknown": 0
    },
    "job_status": "complete"
}
```

### Download Job Results

Request:

```json=
GET /v0/bulk/1/result
```

Expected response:

https://gist.github.com/amaurym/e6a4109a2b2c9baa42806f6953b12fb3#file-reacher_bulk_qa_results-csv

## Further Improvements

This improvements are to be done in future iterations of this feature, in separate Pull Requests.

### 1. Remote URL for POST /v0/bulk

Using a remote URL allows you to host the file and provide us with a direct link to it. The file should be a list of emails separated by line breaks or a standard CSV file. We support most common file transfer protocols and their authentication mechanisms. When using a URL that requires authentication be sure to pass the username and password in the URI string.

Request:

```json=
{
    "input_type": "remote_url",
    "input": "https://mydomain.com/my_file.csv",
    // All fields below are optional
    "proxy": {
        "host": "my.proxy.com",
        "port": "9080"
    },
    "hello_name": "my.domain.com",
    "from_email": "me@my.domain.com",
    "smtp_port": 25
}
```

Contents of `my_file.csv` is a list of emails, one email per line:

```csv=
support@reacher.email
invalid@reacher.email
...
```

Response:

```json=
{
    "job_id": 150970
}
```

### 2. Domain-Specific Message Rate Limiting

Original idea by @z3d3m0n.

```json=
{
    "limits": {
        // "gmail.com" is the top domain of the MX servers.
        "gmail.com": {
            // Max number of concurrent smtp connections. Default should be nice.
            "max_smtp_out": 2,
            // Max number of messages per SMTP connection.
            "max-msg-per-connection": 500,
            // Max number of errors per SMTP connection, to avoid 'too long without data command' error.
            "max-errors-per-connection": 10,
            // Max msg rate, in the format `{number}/{s|m|h}`.
            "max_msg_rate": "9/m",
            "errors":[
                {
                    "code": 421,
                    "SERVICE NOT AVAILABLE"
                }
            ]
        },
    }
}
```

### 3. Download Results as CSV

Add a `?format=csv` query param for downloading the results as CSV:

```
/v0/bulk/<id>/download?format=csv
```

The endpoint returns an `application/octet-stream` containing the job data as a CSV file.

The CSV file looks like:

```csv=
input,is_reachable,misc.is_disposable,misc.is_role_account,misc.error,mx.accepts_mail,mx.records,mx.error,smtp.can_connect_smtp,smtp.has_full_inbox,smtp.is_catch_all,smtp.is_deliverable,smtp.is_disabled,smtp.error,syntax.domain,syntax.is_valid_syntax,syntax.username,syntax.error
someone@gmail.com,invalid,false,false,,true,"alt3.gmail-smtp-in.l.google.com.,gmail-smtp-in.l.google.com.",,true,false,false,false,true,,gmail.com,true,someone,
```

Note how the `mx.records` string array field is encoded as `"<mx1>,<mx2>,<...>"`.

### 4. `DELETE /v0/bulk/<job_ib>`: cancel a job.

### 5. Prune PGSQL db with old email results
