# From https://shaneutt.com/blog/rust-fast-small-docker-image-builds/

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM messense/rust-musl-cross:x86_64-musl as cargo-build

WORKDIR /usr/src/reacher

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/reacher*

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --bin reacher_backend --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 reacher

RUN adduser -D -s /bin/sh -u 1000 -G reacher reacher

WORKDIR /home/reacher/bin/

# Install glibc
# https://github.com/sgerrand/alpine-pkg-glibc
RUN wget -q -O /etc/apk/keys/sgerrand.rsa.pub https://alpine-pkgs.sgerrand.com/sgerrand.rsa.pub && \
    wget https://github.com/sgerrand/alpine-pkg-glibc/releases/download/2.35-r0/glibc-2.35-r0.apk && \
    apk add glibc-2.35-r0.apk

# We need Firefox if we actually want to use GeckoDriver
RUN apk add firefox-esr

# Then install GeckoDriver
ARG GECKODRIVER_VERSION=v0.31.0
RUN wget https://github.com/mozilla/geckodriver/releases/download/$GECKODRIVER_VERSION/geckodriver-$GECKODRIVER_VERSION-linux64.tar.gz && \
    tar -zxf geckodriver-$GECKODRIVER_VERSION-linux64.tar.gz -C /usr/bin

COPY --from=cargo-build /usr/src/reacher/target/x86_64-unknown-linux-musl/release/reacher_backend .
COPY --from=cargo-build /usr/src/reacher/docker.sh .

RUN chown reacher:reacher reacher_backend
RUN chown reacher:reacher docker.sh

USER reacher

ENV RUST_LOG=reacher=info
ENV RCH_HTTP_HOST=0.0.0.0
ENV PORT=8080
ENV RCH_HOTMAIL_USE_HEADLESS=http://localhost:4444
# Bulk verification is disabled by default. Set to 1 to enable it.
ENV RCH_ENABLE_BULK=0

EXPOSE 8080

CMD ["./docker.sh"]
