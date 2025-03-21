#!/bin/bash

set -euxo pipefail

# Ensure the book builds
mdbook build book

# Template builds for RP2040, RP2350 and nRF52840
rustup target add \
    thumbv6m-none-eabi \
    thumbv7em-none-eabihf \
    thumbv8m.main-none-eabihf 

# API ICD
cargo check \
    --manifest-path crates/poststation-api-icd/Cargo.toml \
    --no-default-features \
    --profile ci
cargo check \
    --manifest-path crates/poststation-api-icd/Cargo.toml \
    --all-features \
    --profile ci

# SIM ICD
cargo check \
    --manifest-path crates/poststation-sim-icd/Cargo.toml \
    --profile ci

# FW ICD
cargo check \
    --manifest-path crates/poststation-fw-icd/Cargo.toml \
    --all-features \
    --profile ci

cargo check \
    --manifest-path crates/poststation-fw-icd/Cargo.toml \
    --no-default-features \
    --target thumbv6m-none-eabi \
    --profile ci

# SDK crate
cargo check \
    --manifest-path tools/poststation-sdk/Cargo.toml \
    --profile ci

# CLI tool
cargo build \
    --manifest-path tools/poststation-cli/Cargo.toml \
    --profile ci

## Templates

# ICD
cargo check \
    --manifest-path templates/icd/Cargo.toml \
    --features="use-std" \
    --profile ci

cargo check \
    --manifest-path  templates/icd/Cargo.toml \
    --no-default-features \
    --target thumbv6m-none-eabi \
    --profile ci

# RP2040
cargo build \
    --manifest-path templates/rp2040/Cargo.toml \
    --target thumbv6m-none-eabi \
    --profile ci

# nRF52840
cargo build \
    --manifest-path templates/nrf52840/Cargo.toml \
    --target thumbv7em-none-eabihf \
    --profile ci


## Examples

### waveshare-rp2040-keyboard-3
cargo check \
    --manifest-path examples/waveshare-rp2040-keyboard-3/icd/Cargo.toml \
    --features="use-std" \
    --profile ci

cargo check \
    --manifest-path examples/waveshare-rp2040-keyboard-3/icd/Cargo.toml \
    --no-default-features \
    --target thumbv6m-none-eabi \
    --profile ci

cargo build \
    --manifest-path examples/waveshare-rp2040-keyboard-3/rp2040/Cargo.toml \
    --target thumbv6m-none-eabi \
    --profile ci

cargo build \
    --manifest-path examples/waveshare-rp2040-keyboard-3/demo/Cargo.toml \
    --profile ci

### i2c-passthru
cargo check \
    --manifest-path examples/i2c-passthru/icd/Cargo.toml \
    --features="use-std" \
    --profile ci

cargo check \
    --manifest-path examples/i2c-passthru/icd/Cargo.toml \
    --no-default-features \
    --target thumbv6m-none-eabi \
    --profile ci

cargo build \
    --manifest-path examples/i2c-passthru/rp2040/Cargo.toml \
    --target thumbv6m-none-eabi \
    --profile ci

cargo build \
    --manifest-path examples/i2c-passthru/rp2350/Cargo.toml \
    --target thumbv8m.main-none-eabihf \
    --profile ci

cargo build \
    --manifest-path examples/i2c-passthru/demo/Cargo.toml \
    --profile ci
