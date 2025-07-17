#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Define target platforms
LINUX_TARGET="x86_64-unknown-linux-musl"
WINDOWS_TARGET="x86_64-pc-windows-gnu"
# Install the required Rust targets
rustup target add "$LINUX_TARGET"
rustup target add "$WINDOWS_TARGET"

# Create a directory for the pre-compiled binaries
OUTPUT_DIR="precompiled_binaries"
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# --- Build for Linux ---
echo "Building for Linux..."
cargo build --release --target "$LINUX_TARGET" --package otp-cli
cargo build --release --target "$LINUX_TARGET" --package otp-web

# Strip the Linux binaries to reduce size and move them to the output directory
strip "target/$LINUX_TARGET/release/otp-cli"
strip "target/$LINUX_TARGET/release/otp-web"
mv "target/$LINUX_TARGET/release/otp-cli" "$OUTPUT_DIR/"
mv "target/$LINUX_TARGET/release/otp-web" "$OUTPUT_DIR/"

# --- Build for Windows ---
echo "Building for Windows..."
cargo build --release --target "$WINDOWS_TARGET" --package otp-cli
cargo build --release --target "$WINDOWS_TARGET" --package otp-web

# Move the Windows binaries to the output directory
mv "target/$WINDOWS_TARGET/release/otp-cli.exe" "$OUTPUT_DIR/"
mv "target/$WINDOWS_TARGET/release/otp-web.exe" "$OUTPUT_DIR/"

echo "Pre-compiled binaries are available in the '$OUTPUT_DIR' directory."