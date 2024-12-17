#!/bin/bash

set -euxo pipefail

SERIAL="E66350865F164926"

# Move to bootloader mode
poststation-cli \
    proxy \
    --serial=$SERIAL \
    --path='template/picoboot/reset' \
    --message='{}'

# flash the device
cargo run --release
