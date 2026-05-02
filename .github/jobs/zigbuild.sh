#!/usr/bin/env bash
set -xeuo pipefail

cargo zigbuild --workspace --bins --examples --target "$TARGET"

# Same smoke test as build.sh: run the ctor example (runtime ctor registration).
sleep .1
echo "Running basic example..."
sleep .1
target/${TARGET}/debug/examples/ctor-basic

sleep .1
echo "Running example..."
sleep .1
target/${TARGET}/debug/examples/ctor-example

sleep .1
echo "Running advanced ctor example..."
sleep .1
target/${TARGET}/debug/examples/ctor-advanced

sleep .1
echo "Running link-section example..."
sleep .1
target/${TARGET}/debug/examples/link-section-example
