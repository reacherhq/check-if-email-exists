# Scaling for Production

Reacher's stateless architecture enables efficient horizontal scaling, allowing companies to tailor deployments to their specific needs. Below are the scaling options and pathways for a production setup.

1. [option-1-manage-scaling-yourself.md](option-1-manage-scaling-yourself.md "mention"): Leverage Reacher's statelessness and decide yourself how to deploy Reacher.
2. [option-2-rabbitmq-based-queue-architecture.md](option-2-rabbitmq-based-queue-architecture.md "mention"): Reacher includes a pre-integrated queue system based on [**RabbitMQ**](https://rabbitmq.com), enabling efficient task management and scaling.
3. Option 2: AWS SQS + Lambda (coming soon... or ask [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention"))

Alternative scaling solutions were also explored, chat with [amaury@reacher.email](https://app.gitbook.com/u/F1LnsqPFtfUEGlcILLswbbp5cgk2 "mention") if you want to discuss more.

{% hint style="info" %}
For a high-volume setup, you will most likely need to purchase 3rd-party proxy IPs, see [proxies](../proxies/ "mention").
{% endhint %}

