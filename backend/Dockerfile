# From https://shaneutt.com/blog/rust-fast-small-docker-image-builds/

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM messense/rust-musl-cross:x86_64-musl as cargo-build

WORKDIR /usr/src/reacher

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/reacher*

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 reacher

RUN adduser -D -s /bin/sh -u 1000 -G reacher reacher

WORKDIR /home/reacher/bin/

COPY --from=cargo-build /usr/src/reacher/target/x86_64-unknown-linux-musl/release/reacher_backend .

RUN chown reacher:reacher reacher_backend

USER reacher

ENV RUST_LOG=reacher=info
ENV RCH_HTTP_HOST=0.0.0.0
ENV PORT=8080
# Bulk verification is disabled by default. Set to 1 to enable it.
ENV RCH_ENABLE_BULK=0

EXPOSE 8080

CMD ["./reacher_backend"]
