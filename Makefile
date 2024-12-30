###############################################################################
# Run
###############################################################################

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

# Generate the changelog using the conventional-changelog tool.
# As a hack, we delete all tags that are not beta tags, so that the changelog
# only contains the vX.X.X tags. See:
# https://github.com/conventional-changelog/standard-version/issues/818
#
# To have those tags back locally, run `git fetch --tags`.
.PHONY: changelog
changelog:
	git tag | grep -E '(beta|backend|worker)' | xargs git tag -d
	echo "# Changelog" > CHANGELOG.md
	echo "" >> CHANGELOG.md
	echo "All notable changes to this project will be documented in this file. The changes in this project follow [Convention Commits](https://www.conventionalcommits.org/en/v1.0.0/)." >> CHANGELOG.md
	echo "" >> CHANGELOG.md
	conventional-changelog -p angular -r 0 >> CHANGELOG.md

###############################################################################
# Update lists
###############################################################################

.PHONY: update-role-accounts
update-role-accounts:
# License is MIT.
	curl https://raw.githubusercontent.com/mixmaxhq/role-based-email-addresses/refs/heads/master/index.js -o core/src/misc/roles.txt
# Remove first line, last line, and all ' and , characters
	sed -i.bak '1d' core/src/misc/roles.txt && rm core/src/misc/roles.txt.bak
	sed -i.bak '$$d' core/src/misc/roles.txt && rm core/src/misc/roles.txt.bak
	sed -i.bak 's/['\'', ]//g' core/src/misc/roles.txt && rm core/src/misc/roles.txt.bak


.PHONY: update-free-email-providers
update-free-email-providers:
# License is MIT.
	curl https://raw.githubusercontent.com/ihmpavel/free-email-domains-list/refs/heads/master/data/data.txt -o core/src/misc/b2c.txt