#!/usr/bin/env bash

# Install script of Reacher Backend on an OVH debian 11 server.
# As a postinstall, this script is meant to be run once, but for convenience,
# it's actually idempotent.

# Fail early.
set -e

# You can change the default values of these variables inline here, or by
# setting them in the environment before running this script, e.g.:
# RCH_BACKEND_NAME="my-own-name" ./debian11.sh

# An unique identifier for the backend.
RCH_BACKEND_NAME=${RCH_BACKEND_NAME:-"backend1.mycompany.com"}
# Docker Hub tag for reacherhq/backend.
RCH_VERSION=${RCH_VERSION:-"v0.7.0"}
# Optional: Send bug reports to a Sentry.io dashboard.
RCH_SENTRY_DSN=${RCH_SENTRY_DSN:-}
# Protect the backend from the public via a `x-reacher-secret` header.
RCH_HEADER_SECRET=${RCH_HEADER_SECRET:-}
# For the "FROM" field in emails.
RCH_FROM_EMAIL=${RCH_FROM_EMAIL:-"hello@mycompany.com"}
# For the "EHLO" field in emails. This should ideally match the server's
# reverse DNS entry for optimal results.
RCH_HELLO_NAME=${RCH_HELLO_NAME:-"backend1.mycompany.com"}
# Timeout for SMTP connections in seconds.
RCH_SMTP_TIMEOUT=${RCH_SMTP_TIMEOUT:-"90"}
# Logging. Setup to "debug" to show all logs.
RUST_LOG=${RUST_LOG:-"info"}

echo "Installing Reacher backend $RCH_VERSION on host $RCH_BACKEND_NAME..."

# Install Docker
# https://docs.docker.com/engine/install/debian/
sudo apt-get update
sudo apt-get upgrade --yes
sudo apt-get install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release \
    --yes
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg --yes
echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
    $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-compose-plugin --yes

# Create `docker` group
# https://docs.docker.com/engine/install/linux-postinstall/
getent group docker || sudo groupadd docker
sudo usermod -aG docker debian
# Reload users and groups, see
# https://superuser.com/questions/272061/reload-a-linux-users-group-assignments-without-logging-out
sudo su - $USER << EOF

# Stop all previous docker containers and images
docker stop reacher_backend
docker rm reacher_backend

# Run the backend
docker run -d \
    -e RUST_LOG=$RUST_LOG \
    -e RCH_BACKEND_NAME=$RCH_BACKEND_NAME \
    -e RCH_SENTRY_DSN=$RCH_SENTRY_DSN \
    -e RCH_HEADER_SECRET=$RCH_HEADER_SECRET \
    -e RCH_FROM_EMAIL=$RCH_FROM_EMAIL \
    -e RCH_HELLO_NAME=$RCH_HELLO_NAME \
    -e RCH_SMTP_TIMEOUT=$RCH_SMTP_TIMEOUT \
    -p 80:8080 \
    --name reacher_backend \
    reacherhq/backend:$RCH_VERSION

echo "Everything set. You can close this terminal."
EOF
