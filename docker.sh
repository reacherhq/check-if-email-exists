#!/bin/ash

# https://docs.docker.com/config/containers/multi-service_container/
MOZ_HEADLESS=1 geckodriver &
./reacher_backend
