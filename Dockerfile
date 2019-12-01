FROM rust:1.39

WORKDIR /ciee
COPY . .
RUN cargo build --release

EXPOSE 3000

CMD ["./target/release/check_if_email_exists", "--http"]
