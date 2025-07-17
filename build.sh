#!/bin/bash

set -e

# Build the CLI for Linux
echo "Building CLI for Linux..."
cargo build --release --target x86_64-unknown-linux-musl --package otp-cli
strip target/x86_64-unknown-linux-musl/release/otp-cli

# Build the web server for Linux
echo "Building web server for Linux..."
cargo build --release --target x86_64-unknown-linux-musl --package otp-web
strip target/x86_64-unknown-linux-musl/release/otp-web

echo "Build complete."