#!/usr/bin/env bash

# This script is used to benchmark the throughput of the server. It assumes 3
# things:
# 1. The server is running on localhost:8080.
# 2. The wrk tool is installed. See https://github.com/wg/wrk.
# 3. The bench_throughput.lua script is in the same folder as this script.

# Make sure bench_throughput.lua exists in current folder.
if [ ! -f bench_throughput.lua ]; then
    echo "bench_throughput.lua not found!"
    exit 1
fi

wrk -t12 -c400 -d20s -s bench_throughput.lua http://localhost:8080/v0/check_email