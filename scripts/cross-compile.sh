#!/usr/bin/env bash
# Copyright (C) 2025 Mathew Robinson <chasinglogic@gmail.com>
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free Software
# Foundation, version 3.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program. If not, see <https://www.gnu.org/licenses/>.
#

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