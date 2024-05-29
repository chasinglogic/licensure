#!/usr/bin/env bash

if [[ ! -x $(which cross 2>/dev/null) ]]; then
    echo "You need to install cross to use this script:"
    echo "    https://github.com/cross-rs/cross"
    exit 1
fi

if [[ ! -x $(which docker 2>/dev/null) ]]; then
    echo "You need to install docker to use this script:"
    echo "    https://docker.com"
    exit 1
fi

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "arm-unknown-linux-gnueabihf"
)

for target in "${TARGETS[@]}"; do
    cross build --release --target "$target"
done