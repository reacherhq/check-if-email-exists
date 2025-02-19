# Option 1: Manage scaling yourself

Reacher is stateless by design. This means that you can spawn instances of Reacher concurrently, and they don't need coordinate to perform email verifications.

However, in most cases, some coordination is desired. For example, to avoid IPs being blacklisted, we might want to limit to the number of spawned instances at any time (concurrency) or during a given period of time (throttling). See the last section in [proxies](../proxies/ "mention") for some actual numbers for these settings.

The best scaling solution for you is the one that fits into your architecture. Below are several scaling solutions, including both traditional and advanced approaches. They are often not mutually exclusive.

#### 1. Install Reacher on dedicated servers (proxies optional).

* **Description**: Install Reacher on dedicated servers. This setup is straightforward and allows you to manage IP quality directly.
* **Pros**:
  * Complete control over IP maintenance (if desired).
  * Independence from 3rd-party proxies.
* **Cons**:
  * Limited scalability compared to cloud-based solutions.
  * Higher initial infrastructure cost.

#### 2. Pure serverless architecture.

* **Description**: Deploy Reacher as a stateless function using serverless platforms like AWS Lambda or Google Cloud Functions.
* **Pros**:
  * Infinite horizontal scaling.
  * No infrastructure management.
* **Cons**:
  * Complexity in managing rate-limiting and proxy usage (see those limits in [proxies](../proxies/ "mention")).
  * Higher cost for large-scale, constant usage.

#### 3. **Amazon SQS with Worker Instances.**

* **Description**: Use [Amazon SQS](https://aws.amazon.com/sqs/) (or similar message queues) to distribute email verification tasks to worker instances running Reacher.
* **Architecture**:
  * Task requests are queued in SQS (or another broker).
  * Reacher workers (Docker containers or serverless functions) poll the queue to process tasks.
* **Pros**:
  * Efficient task distribution.
  * Built-in concurrency control via SQS throttling.
* **Cons**:
  * Requires managing worker instances.

This solution is explor more in details in [option-2-rabbitmq-based-queue-architecture.md](option-2-rabbitmq-based-queue-architecture.md "mention"), using [RabbitMQ](https://rabbitmq.com) instead of Amazon SQS.

#### **4. Kubernetes (k8s) Clusters**

* **Description**: Run Reacher instances as pods within a Kubernetes cluster for high availability and scalability.
* **Architecture**:
  * Use Kubernetes' Horizontal Pod Autoscaler (HPA) to scale Reacher pods based on task load.
  * Integrate with RabbitMQ for task queuing and distribution (see [option-2-rabbitmq-based-queue-architecture.md](option-2-rabbitmq-based-queue-architecture.md "mention"))
* **Pros**:
  * Highly scalable and resilient.
  * Centralized monitoring and orchestration.
* **Cons**:
  * Higher operational complexity.
  * Requires Kubernetes expertise.
