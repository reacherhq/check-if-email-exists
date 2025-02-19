# SaaS vs Self-Host

When using Reacher, you can choose between two options: the SaaS version hosted by Reacher or self-hosting the service on your infrastructure. Each option offers distinct advantages depending on your needs, technical resources, and desired level of control.

## What is Reacher SaaS?

Reacher SaaS is the cloud-hosted version of the service, accessible at [https://app.reacher.email](https://app.reacher.email). It handles all infrastructure, updates, and scaling for you. You can verify emails directly via the Reacher Dashboard without managing technical details.

## What is Self-Hosting Reacher?

Reacher is also available for self-hosting, enabling you to run the service on your own servers. This approach provides full control over the environment and data while leveraging the same verification engine as Reacher SaaS. Reacher is designed to make self-hosting straightforward and efficient.

{% hint style="success" %}
Reacher's goal is to make Self-Hosting easy. You can [install.md](install.md "mention") as part of your **Commercial License Trial**.
{% endhint %}

## Key Differences

| Feature               | SaaS                                   | Self-Hosting                                                                              |
| --------------------- | -------------------------------------- | ----------------------------------------------------------------------------------------- |
| **Volume**            | Limited to 10k verifications per month | Unlimited verifications                                                                   |
| **Cost**              | Monthly subscription                   | Monthly subscription + server costs (lower at scale) + [proxy](proxies/) costs (optional) |
| **Setup Time**        | Instant                                | Requires installation and setup                                                           |
| **Maintenance**       | Fully managed by Reacher               | Managed by your IT team                                                                   |
| **Data Ownership**    | Data stored on Reacher servers         | Full ownership, no data is sent to Reacher                                                |
| **Bulk Verification** | Not supported                          | Supported, see [bulk.md](../advanced/migrations/bulk.md "mention")                        |
