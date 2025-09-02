#!/bin/bash

# Build script for JSON-only iMessage exporter
# This creates a minimal binary that only supports JSON export

set -e

echo "Building JSON-only iMessage exporter..."

# Create output directory
mkdir -p output

# Build the JSON-only version
echo "Building with Cargo.json-only.toml..."
cargo build --manifest-path ./Cargo.json-only.toml --release

# Copy the binary to output directory
cp target/release/imessage-json-exporter output/

# Create a compressed archive
cd target/release
tar -czf ../../output/imessage-json-exporter.tar.gz imessage-json-exporter
cd ../..

echo "Build complete!"
echo "Binary location: output/imessage-json-exporter"
echo "Archive location: output/imessage-json-exporter.tar.gz"

# Show binary info
echo ""
echo "Binary information:"
ls -lh output/imessage-json-exporter
file output/imessage-json-exporter

echo ""
echo "To run the JSON exporter:"
echo "./output/imessage-json-exporter --help"
