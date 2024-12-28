FROM rust:1.82 as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev musl-tools

# Set the working directory
WORKDIR /usr/src/app

# Copy the Rust project files
COPY . .

# Compile the application for the Lambda runtime (x86_64-unknown-linux-musl)
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

# Create the runtime container
FROM public.ecr.aws/lambda/provided:al2

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/reacher_sqs /var/task/bootstrap

# Set the Lambda handler (entry point)
CMD ["bootstrap"]
