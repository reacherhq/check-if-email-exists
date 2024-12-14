# Run the backend without worker mode, i.e. only enabling single-shot
# verifications via the /v1/check_email endpoint.
.PHONY: run
run:
	cd backend && cargo run --bin reacher_backend


# Run the backend with worker mode on. This enables the /v1/bulk endpoints.
# Make sure to have a Postgres DB and a RabbitMQ instance running.
.PHONY: run-with-worker
run-with-worker: export RCH__WORKER__ENABLE=true
run-with-worker: export RCH__WORKER__RABBITMQ__URL=amqp://guest:guest@localhost:5672
run-with-worker: export RCH__STORAGE__POSTGRES__DB_URL=postgresql://localhost/reacherdb
run-with-worker: run

.PHONY: run-with-commercial-license-trial
run-with-commercial-license-trial: export RCH__COMMERCIAL_LICENSE_TRIAL__URL=http://localhost:3000/api/v1/commercial_license_trial
run-with-commercial-license-trial: run