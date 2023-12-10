#!/bin/ash

# https://docs.docker.com/config/containers/multi-service_container/
chromedriver &
./reacher_worker
