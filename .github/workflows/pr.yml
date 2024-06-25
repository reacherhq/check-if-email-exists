# Based on https://github.com/actions-rs/meta/blob/master/recipes/quickstart.md

name: pr

on:
  pull_request:
  push:
    branches:
      - master

# For backend, run builds and tests without sqlx's compile time query checks
# https://github.com/launchbadge/sqlx/blob/master/sqlx-cli/README.md#force-building-in-offline-mode
env:
  SQLX_OFFLINE: true

jobs:
  # Cargo check.
  check:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout sources
        uses: actions/checkout@v2

      - name: Install stable toolchain
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true

      - name: Run cargo check
        uses: actions-rs/cargo@v1
        with:
          command: check
          args: --all

  # Cargo test.
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout sources
        uses: actions/checkout@v2

      - name: Install stable toolchain
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true

      - name: Run cargo test
        uses: actions-rs/cargo@v1
        with:
          command: test
          args: --all

  # Cargo fmt and clippy
  lints:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout sources
        uses: actions/checkout@v2

      - name: Install stable toolchain
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          components: rustfmt, clippy

      - name: Run cargo fmt
        uses: actions-rs/cargo@v1
        with:
          command: fmt
          args: --all -- --check

      - name: Run cargo clippy
        uses: actions-rs/cargo@v1
        with:
          command: clippy
          args: --all -- -A deprecated
