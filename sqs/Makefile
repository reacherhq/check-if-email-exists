TAG = beta

.PHONY: docker_deploy
docker_deploy:
	cd .. && docker buildx build --platform linux/amd64 -t 430118836964.dkr.ecr.eu-west-3.amazonaws.com/reacherhq/sqs:$(TAG) -f sqs/Dockerfile .
