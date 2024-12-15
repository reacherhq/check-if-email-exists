#!/bin/ash

# This is the Dockerfile's entrypoint script.
# https://docs.docker.com/config/containers/multi-service_container/

# Function to terminate processes
cleanup() {
    echo "Shutting down..."
    kill $(jobs -p)
    exit 0
}

# Set up signal handling
trap cleanup SIGTERM SIGINT

# Start chromedriver in background
chromedriver &

# Start reacher backend in background
./reacher_backend &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?
