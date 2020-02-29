FROM sitkevij/alpine-rust

# `ciee` stands for check-if-email-exists
WORKDIR /ciee
ENV CIEE_VERSION 0.6.4

COPY . .

RUN cargo build --release

CMD ["./target/release/check_if_email_exists", "--http", "--http-host", "0.0.0.0"]
