# Commercial License Trial

The Commercial License Trial allows you to test the self-hosted software for a limited period of time, for **internal testing** and **non-commercial purposes**.

## Features and Limitations

As part of the Commercial License Trial, you'll receive a Dockerfile designed to enable quick setup for email verifications. Below are its key features and limitations:

* **Built-in Proxy Configuration**: we embed [**Proxy4Smtp**](https://www.proxy4smtp.com), a 3rd-party proxy with carefully maintained IPs optimized for SMTP verifications. All verifications using the Dockerfile go through this proxy. This ensures reliable email verification even in cloud environments with restricted SMTP access. Learn more in [proxies](../proxies/ "mention").
* **Daily Verification Limit**: capped at **60 per minute** and **10,000 per day.**
* **Usage Tracking**: verification results are anonymized and sent back to Reacher, and used to monitor daily usage and detect potential abuse.
* **For testing purposes only**. The Dockerfile can only be used internally, for testing purposes, and can in no case be used in production environments for commercial applications.

{% hint style="danger" %}
All abuse of the Commercial License Trial will result in an immediate account ban.
{% endhint %}

## Get Started

To start your Commercial License Trial, sign up on [https://reacher.email](https://app.reacher.email/en/signup). Then, navigate to the "Commercial License Trial" tab of the Reacher Dashboard ([go there directly](https://app.reacher.email/en/dashboard/commercial_license)) and follow instructions.

## After the Trial

The Commercial License Trial cannot be used for full-scale production use or commercial applications. Once you've tested Reacher extensively, purchase a Commercial License.

Once you purchased the Commercial License, you'll need to perform the two following steps:

1. Replace the Docker image from `reacherhq/commercial-license-trial`to `reacherhq/backend`. This will give you access to an unrestricted Dockerfile to perform unlimited verifications.
2. (If applicable) Configure proxy setting on the Docker container, via the `RCH__PROXY__*`environment variables. See [proxies](../proxies/ "mention")for more information.
