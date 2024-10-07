[![Docker](https://img.shields.io/docker/v/reacherhq/worker?color=0db7ed&label=docker&sort=date)](https://hub.docker.com/r/reacherhq/worker)
[![Actions Status](https://github.com/reacherhq/check-if-email-exists/workflows/pr/badge.svg)](https://github.com/reacherhq/check-if-email-exists/actions)

<br /><br />

<p align="center"><img align="center" src="https://storage.googleapis.com/saasify-uploads-prod/696e287ad79f0e0352bc201b36d701849f7d55e7.svg" height="96" alt="reacher" /></p>
<h1 align="center">⚙️ Reacher Task Queue</h1>
<h4 align="center">Architecture for distributing tasks from a queue to multiple Reacher workers.</h4>

<br /><br />

## Get Started

The easiest way to get started is with Docker Compose. This setup runs one queue and two Reacher Workers. From the **root** folder, run:

```bash
docker compose up --build
# Wait for everything to run correctly
```

Then, publish some email verification requests to the RabbitMQ queue:

```bash
cd worker
cargo run -p reacher_worker --bin publish_message <email1> <email2> ...
```

The Docker Compose logs will show the two workers sharing the email verifications. You may ask amaury@reacher.email about other methods for publishing messages to this queue (JavaScript, Python...).
