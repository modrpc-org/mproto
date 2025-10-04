#!/bin/sh

set -e

# Change to mproto/integ-test/ directory
cd $(dirname -- "$( readlink -f -- "$0"; )")
pwd

cd ../crates/mprotoc/
cargo build --release
cd -

RUST_BACKTRACE=1 ../target/release/mprotoc proto/test.mproto \
    --package -l rust -n test-mproto
RUST_BACKTRACE=1 ../target/release/mprotoc proto/test.mproto \
    --package -l typescript -n test-mproto

# Attempt to compile rust package
cd test-mproto/rust/
cargo build
cd -

# Attempt to compile typescript package
cd test-mproto/typescript/
npm install .
npm run build
cd -

echo "mproto integ tests passed."
