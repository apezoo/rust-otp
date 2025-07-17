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
echo "Cleaning up old builds and creating output directory..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# --- Build all workspace binaries for Linux ---
echo "Building all workspace binaries for Linux ($LINUX_TARGET)..."
cargo build --release --workspace --target "$LINUX_TARGET"

# --- Build all workspace binaries for Windows ---
echo "Building all workspace binaries for Windows ($WINDOWS_TARGET)..."
cargo build --release --workspace --target "$WINDOWS_TARGET"

# --- Post-build processing ---
echo "Stripping and organizing binaries..."

# Strip and move Linux binaries
for bin in target/"$LINUX_TARGET"/release/otp-*; do
    if [[ -f "$bin" && ! -d "$bin" ]]; then
        strip "$bin"
        mv "$bin" "$OUTPUT_DIR/"
    fi
done

# Move Windows binaries
for bin in target/"$WINDOWS_TARGET"/release/otp-*.exe; do
    if [[ -f "$bin" ]]; then
        mv "$bin" "$OUTPUT_DIR/"
    fi
done

echo "Pre-compiled binaries are available in the '$OUTPUT_DIR' directory."