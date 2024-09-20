# SaaS vs Self-Host

When using Reacher, you have two options: using the **SaaS version** provided by Reacher or **self-hosting** the service on your own infrastructure. Each approach has unique advantages depending on your specific needs, technical resources, and how much control you want over the service.

## What is Reacher SaaS?

Reacher SaaS is the cloud-hosted version of the service, which is available at [https://app.reacher.email](https://app.reacher.email), where everything is managed for you. You can verify emails directly via the Reacher Dashboard without needing to worry about infrastructure, updates, or scaling.

## What is Self-Hosting Reacher?

Reacher is also designed for self-hosting, allowing you to run the service on your own servers. This option gives you full control over the environment and data while using the same powerful verification engine that Reacher SaaS provides.

{% hint style="success" %}
Reacher's goal is to make Self-Hosting easy. You can [install.md](install.md "mention").
{% endhint %}

## Key Differences

| Feature               | SaaS                                   | Self-Hosting                                                                     |
| --------------------- | -------------------------------------- | -------------------------------------------------------------------------------- |
| **Volume**            | Limited to 10k verifications per month | Unlimited verifications                                                          |
| **Cost**              | Monthly subscription                   | Monthly subscription + your own server costs. Amortized costs for large volumes. |
| **Setup Time**        | Instant                                | Requires installation and setup                                                  |
| **Maintenance**       | Fully managed by Reacher               | Managed by your IT team                                                          |
| **Data Ownership**    | Data stored on Reacher servers         | Full ownership, no data is sent to Reacher                                       |
| **Bulk Verification** | No                                     | Yes, see [bulk.md](bulk.md "mention")                                            |
