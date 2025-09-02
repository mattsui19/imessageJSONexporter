# iMessage JSON Exporter (Minimal Build)

This is a minimal build of the iMessage exporter that **only supports JSON export format**. It removes all HTML and TXT export functionality to create a smaller, focused binary.

## Quick Build

```bash
# Make the build script executable
chmod +x build-json-only.sh

# Build the JSON-only version
./build-json-only.sh
```

## Manual Build

```bash
# Build using the JSON-only Cargo.toml
cargo build --manifest-path Cargo.json-only.toml --release

# The binary will be at: target/release/imessage-json-exporter
```

## What's Included

✅ **JSON Export** - Full JSON message export functionality  
✅ **Database Support** - SQLite database reading  
✅ **Attachment Handling** - File attachment metadata  
✅ **CLI Interface** - Command-line options  
✅ **Progress Tracking** - Export progress display  

❌ **HTML Export** - Removed  
❌ **TXT Export** - Removed  
❌ **HTML Templates** - Removed  
❌ **Text Formatting** - Removed  

## Usage

```bash
# Basic JSON export
./imessage-json-exporter --export-path ./output

# With custom database path
./imessage-json-exporter --db-path /path/to/chat.db --export-path ./output

# Run diagnostics
./imessage-json-exporter --diagnostics

# Show help
./imessage-json-exporter --help
```

## Output Format

The JSON exporter creates structured output like:

```json
{
  "timestamp": "2022-05-17 20:29:42",
  "sender": "Me",
  "contents": "Hello world",
  "attachments": [],
  "readtime": null,
  "is_from_me": true,
  "guid": "message-guid-here"
}
```

## File Structure

```
src/
├── main.json-only.rs          # Main entry point (JSON only)
├── app/
│   ├── mod.json-only.rs       # App module (JSON only)
│   ├── export_type.json-only.rs # Export type (JSON only)
│   └── options.json-only.rs   # CLI options (JSON only)
└── exporters/
    ├── mod.json-only.rs       # Exporters module (JSON only)
    ├── exporter.rs            # Base exporter trait
    └── json.rs               # JSON exporter implementation
```

## Benefits of JSON-Only Build

- **Smaller binary size** - No HTML/TXT code
- **Faster compilation** - Fewer dependencies
- **Focused functionality** - Only JSON export
- **Easier maintenance** - Simpler codebase
- **Better performance** - No unused code paths

## Testing

```bash
# Run all tests
cargo test --manifest-path Cargo.json-only.toml

# Run specific test modules
cargo test --manifest-path Cargo.json-only.toml export_type
cargo test --manifest-path Cargo.json-only.toml json
```

## Dependencies

The JSON-only build only includes essential dependencies:
- `clap` - CLI argument parsing
- `imessage-database` - Database reading library
- `serde_json` - JSON serialization
- `rusqlite` - SQLite database access
- `indicatif` - Progress bars
- `filetime`, `fdlimit`, `fs2` - File system utilities
