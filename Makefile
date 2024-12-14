# Run the backend without worker mode, i.e. only enabling single-shot
# verifications via the /v1/check_email endpoint.
.PHONY: run-without-worker
run-without-worker:
	cd backend && cargo run --bin reacher_backend


# Run the backend with worker mode on. This enables the /v1/bulk endpoints.
# Make sure to have a Postgres DB and a RabbitMQ instance running.
.PHONY: run-with-worker
run-with-worker: export RCH__WORKER__ENABLE=true
run-with-worker: export RCH__WORKER__RABBITMQ__URL=amqp://guest:guest@localhost:5672
run-with-worker: export RCH__STORAGE__POSTGRES__DB_URL=postgresql://localhost/reacherdb
run-with-worker: run-without-worker
