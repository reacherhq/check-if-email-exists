FROM ubuntu

# `ciee` stands for check-if-email-exists
WORKDIR /ciee
ENV CIEE_VERSION 0.6.0

# Install needed libraries
RUN apt-get update && \
  apt-get install -y wget

# Download the binary from Github
RUN wget https://github.com/amaurymartiny/check-if-email-exists/releases/download/v${CIEE_VERSION}/check-if-email-exists-v${CIEE_VERSION}-x86_64-unknown-linux-gnu.tar.gz \
    && tar -xvzf check-if-email-exists-v${CIEE_VERSION}-x86_64-unknown-linux-gnu.tar.gz

RUN ls -la

CMD ["./check_if_email_exists", "--http"]
