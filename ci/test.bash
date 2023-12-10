#!/usr/bin/env bash
# Script for building your rust projects.
set -e

source ci/common.bash

# $1 {path} = Path to cross/cargo executable
CROSS=$1
# $1 {string} = <Target Triple>
TARGET_TRIPLE=$2

required_arg $CROSS 'CROSS'
required_arg $TARGET_TRIPLE '<Target Triple>'

# reacher_worker doesn't compile on windows because of this bug:
# https://github.com/amqp-rs/reactor-trait/issues/1
# So we only build the check_if_email_exists binary for Windows.
$CROSS test --bin check_if_email_exists --target $TARGET_TRIPLE
$CROSS test --bin check_if_email_exists --target $TARGET_TRIPLE --all-features
