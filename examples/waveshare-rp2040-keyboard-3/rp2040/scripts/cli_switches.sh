#!/bin/bash

set -euxo pipefail

# You can find your serial using `poststation-cli ls`
SERIAL="E66350865F164926"

poststation-cli \
    listen \
    --serial $SERIAL \
    --path="keyboard/switches"
