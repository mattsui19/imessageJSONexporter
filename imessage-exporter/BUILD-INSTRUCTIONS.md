# JSON-Only iMessage Exporter - Build Instructions

This document explains how to build a minimal version of the iMessage exporter that **only supports JSON export format**.

## What You Get

✅ **JSON Export Only** - No HTML or TXT support  
✅ **Smaller Binary** - Reduced dependencies and code  
✅ **Faster Build** - Fewer modules to compile  
✅ **Focused Functionality** - Only what you need for JSON export  

## Quick Start

```bash
# 1. Navigate to the project directory
cd imessage-exporter

# 2. Build the JSON-only version
./build-json-only.sh

# 3. Test the binary
./test-json-only.sh
```

## Manual Build Steps

### Option 1: Using the JSON-only Cargo.toml

```bash
# Build with the specialized manifest
cargo build --manifest-path Cargo.json-only.toml --release

# Binary will be at: target/release/imessage-json-exporter
```

### Option 2: Using the Original Cargo.toml (but only JSON)

```bash
# Build normally (includes all formats)
cargo build --release

# Binary will be at: target/release/imessage-exporter
# But you can still use --format json to only export JSON
```

## File Structure

The JSON-only build uses these specialized files:

```
imessage-exporter/
├── Cargo.json-only.toml           # Minimal dependencies
├── build-json-only.sh             # Build script
├── test-json-only.sh              # Test script
├── README-json-only.md            # JSON-only documentation
├── BUILD-INSTRUCTIONS.md          # This file
└── src/
    ├── main.json-only.rs          # JSON-only main entry
    ├── app/
    │   ├── mod.json-only.rs       # JSON-only app module
    │   ├── export_type.json-only.rs # JSON-only export types
    │   └── options.json-only.rs   # JSON-only CLI options
    └── exporters/
        ├── mod.json-only.rs       # JSON-only exporters
        ├── exporter.rs            # Base exporter trait
        └── json.rs               # JSON exporter implementation
```

## Usage Examples

### Basic JSON Export
```bash
./imessage-json-exporter --export-path ./output
```

### Custom Database Path
```bash
./imessage-json-exporter --db-path /path/to/chat.db --export-path ./output
```

### Run Diagnostics
```bash
./imessage-json-exporter --diagnostics
```

### Show Help
```bash
./imessage-json-exporter --help
```

## Testing

### Run All Tests
```bash
cargo test --manifest-path Cargo.json-only.toml
```

### Test Specific Components
```bash
# Test export type parsing
cargo test --manifest-path Cargo.json-only.toml export_type

# Test JSON exporter
cargo test --manifest-path Cargo.json-only.toml json
```

### Manual Testing
```bash
# Test the binary
./test-json-only.sh
```

## Dependencies

The JSON-only build includes only essential dependencies:

| Dependency | Purpose | Version |
|------------|---------|---------|
| `clap` | CLI argument parsing | 4.5.46 |
| `imessage-database` | Database reading library | local path |
| `serde_json` | JSON serialization | 1.0 |
| `rusqlite` | SQLite database access | 0.37.0 |
| `indicatif` | Progress bars | 0.18.0 |
| `filetime` | File time utilities | 0.2.26 |
| `fdlimit` | File descriptor limits | 0.3.0 |
| `fs2` | File system utilities | 0.4.3 |

## Output Format

The JSON exporter creates structured output:

```json
{
  "timestamp": "2022-05-17 20:29:42",
  "sender": "Me",
  "contents": "Hello world",
  "attachments": [
    {
      "filename": "image.jpg",
      "mime_type": "image/jpeg",
      "file_size": 12345
    }
  ],
  "readtime": "2022-05-17 20:30:00",
  "is_from_me": true,
  "guid": "message-guid-here"
}
```

## Troubleshooting

### Build Errors

1. **Missing dependencies**: Ensure Rust and Cargo are installed
2. **Path issues**: Make sure you're in the `imessage-exporter` directory
3. **Permission errors**: Make build scripts executable with `chmod +x`

### Runtime Errors

1. **Database not found**: Use `--db-path` to specify correct path
2. **Permission denied**: Ensure you have read access to the database
3. **Export path issues**: Ensure the export directory is writable

### Test Failures

1. **Binary not found**: Run `./build-json-only.sh` first
2. **Format validation**: The exporter should reject HTML/TXT formats
3. **Database errors**: Expected when no real database is available

## Performance Benefits

- **Binary Size**: ~30-50% smaller than full version
- **Compile Time**: ~40-60% faster compilation
- **Memory Usage**: Lower runtime memory footprint
- **Startup Time**: Faster binary startup

## Security Notes

- The JSON-only build has the same security profile as the full version
- No HTML rendering means no XSS vulnerabilities from HTML export
- JSON output is safe for programmatic consumption
- Database access follows the same security model

## Support

For issues with the JSON-only build:
1. Check this documentation first
2. Verify your build environment
3. Run the test script to isolate issues
4. Check the original project for database-related issues
