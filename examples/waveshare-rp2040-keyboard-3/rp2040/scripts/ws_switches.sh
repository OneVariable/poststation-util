#!/bin/bash

set -euxo pipefail

# You can find your serial using `poststation-cli ls`
SERIAL="E66350865F164926"
KEY="BCE2A46EF945B430"

websocat "ws://localhost:4444/api/devices/$SERIAL/listen?path=keyboard/switches&key=$KEY" | jq
