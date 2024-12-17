#!/bin/bash

set -euxo pipefail

# Template builds for RP2040/cortex-m0+
rustup target add \
    thumbv6m-none-eabi

# API ICD
cargo build \
    --manifest-path crates/poststation-api-icd/Cargo.toml \
    --no-default-features \
    --profile ci
cargo build \
    --manifest-path crates/poststation-api-icd/Cargo.toml \
    --all-features \
    --profile ci

# SDK crate
cargo build \
    --manifest-path tools/poststation-sdk/Cargo.toml \
    --profile ci

# CLI tool
cargo build \
    --manifest-path tools/poststation-cli/Cargo.toml \
    --profile ci

## Templates

# ICD
cargo build \
    --manifest-path templates/icd/Cargo.toml \
    --features="use-std" \
    --profile ci

cargo build \
    --manifest-path  templates/icd/Cargo.toml \
    --no-default-features \
    --target thumbv6m-none-eabi \
    --profile ci

# RP2040
cargo build \
    --manifest-path  templates/rp2040/Cargo.toml \
    --target thumbv6m-none-eabi \
    --profile ci
