# Deprecated.
# This Dockerfile is for demo purposes, and should not be used in production.
# For a production-ready web server, please see https://github.com/reacherhq/backend.
# This Dockerfile will be **deprecated** soon, and removed from the
# Docker Hub: https://hub.docker.com/r/reacherhq/check-if-email-exists.
FROM alpine

# `ciee` stands for check-if-email-exists
WORKDIR /ciee
# Fetch latest version
ENV CIEE_VERSION 0.8.24

# Install needed libraries
RUN apk update && \
  apk add --no-cache openssl wget && \
  rm -rf /var/cache/apk/*

# Download the binary from Github
RUN wget https://github.com/reacherhq/check-if-email-exists/releases/download/v${CIEE_VERSION}/check-if-email-exists-v${CIEE_VERSION}-x86_64-unknown-linux-musl.tar.gz \
    && tar -xvzf check-if-email-exists-v${CIEE_VERSION}-x86_64-unknown-linux-musl.tar.gz

CMD ["./check_if_email_exists", "--http", "--http-host", "0.0.0.0"]
