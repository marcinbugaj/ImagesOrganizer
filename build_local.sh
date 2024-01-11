#!/bin/bash

set -e

pushd src/ui
npm install
npm run build
popd

$HOME/.cargo/bin/cargo build --release
$HOME/.cargo/bin/cargo build --release --target x86_64-pc-windows-gnu

