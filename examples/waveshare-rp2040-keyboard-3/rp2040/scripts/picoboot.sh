#!/bin/bash

set -euxo pipefail

# You can find your serial using `poststation-cli ls`
SERIAL="E66350865F164926"

# Move to bootloader mode
poststation-cli \
    proxy \
    --serial=$SERIAL \
    --path='template/picoboot/reset' \
    --message='{}'

# flash the device
cargo run --release
