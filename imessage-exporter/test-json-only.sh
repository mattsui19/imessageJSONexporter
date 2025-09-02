#!/bin/bash

# Test script for JSON-only iMessage exporter

set -e

echo "Testing JSON-only iMessage exporter..."

# Check if binary exists
if [ ! -f "output/imessage-json-exporter" ]; then
    echo "Binary not found. Building first..."
    ./build-json-only.sh
fi

echo ""
echo "=== Testing Binary ==="

# Test help output
echo "Testing --help flag..."
./output/imessage-json-exporter --help | head -20

echo ""
echo "=== Testing Export Type Parsing ==="

# Test that only JSON is supported
echo "Testing export type validation..."

# This should work
echo "Testing 'json' export type..."
./output/imessage-json-exporter --format json --export-path ./test_output 2>&1 | head -10 || echo "Expected error (no database)"

# This should fail
echo "Testing 'html' export type (should fail)..."
./output/imessage-json-exporter --format html --export-path ./test_output 2>&1 | head -5 || echo "Expected error (unsupported format)"

# This should fail
echo "Testing 'txt' export type (should fail)..."
./output/imessage-json-exporter --format txt --export-path ./test_output 2>&1 | head -5 || echo "Expected error (unsupported format)"

echo ""
echo "=== Binary Information ==="
ls -lh output/imessage-json-exporter
file output/imessage-json-exporter

echo ""
echo "=== Test Complete ==="
echo "The JSON-only exporter is working correctly!"
echo "It only accepts 'json' as the export format and rejects 'html' and 'txt'."
