#!/bin/bash

set -euxo pipefail

SERIAL="E66350865F164926"

poststation-cli \
    proxy \
    --serial=$SERIAL \
    --path='keyboard/rgb/set' \
    --message='{"color": {"r": 255, "g": 255, "b": 255}, "position": "One"}'

poststation-cli \
    proxy \
    --serial=$SERIAL \
    --path='keyboard/rgb/set' \
    --message='{"color": {"r": 255, "g": 255, "b": 255}, "position": "Two"}'

poststation-cli \
    proxy \
    --serial=$SERIAL \
    --path='keyboard/rgb/set' \
    --message='{"color": {"r": 255, "g": 255, "b": 255}, "position": "Three"}'
