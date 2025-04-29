# Scaling for Production

Reacher's stateless architecture enables efficient horizontal scaling, allowing companies to tailor deployments to their specific needs. Below are the scaling options and pathways for a production setup.

1. [option-1-manage-scaling-yourself.md](option-1-manage-scaling-yourself.md "mention"): Leverage Reacher's statelessness and decide yourself how to deploy Reacher.
2. [option-2-rabbitmq-based-queue-architecture.md](option-2-rabbitmq-based-queue-architecture.md "mention"): Reacher includes a pre-integrated queue system based on [**RabbitMQ**](https://rabbitmq.com), enabling efficient task management and scaling.
3. Option 2: AWS SQS + Lambda (coming soon... or ask [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention"))

Alternative scaling solutions were also explored, chat with [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention") if you want to discuss more.

## Scaling beyond the Commercial License Trial

The documentation in this section mentions the Dockerfile provided as part of the [commercial-license-trial.md](../licensing/commercial-license-trial.md "mention"), which:

* has a built-in proxy,
* &#x20;limits the number of daily verifications to 10000.

The strategies documented in this section apply both to the Commercial License Trial as well as a high-volume setup beyond this limit.  Once you're ready to transition from the former to the latter, you must:

1. **Purchase a Commercial License**. This will grant you access to an unrestricted Dockerfile.
2. **Purchase 3rd-party proxy IPs**. You will receive configuration details for the proxy to be passed into the unrestricted Dockerfile.

Get in touch with [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention") when you're there.

