# DeaDBeeF Discord Rich Presence Plugin

A Discord Rich Presence plugin for [DeaDBeeF music player](https://deadbeef.sourceforge.io/), written in Rust.

## Features

- 🎵 Display currently playing track information on Discord
- 🎨 Automatic album artwork fetching via MusicBrainz
- ⚙️ Configurable display options
- 🚀 Lightweight and efficient (optimized for minimal size)

## Requirements

- DeaDBeeF music player
- Discord desktop client
- Rust toolchain (for building from source)

## Building

### Prerequisites

Make sure you have Rust installed.  If not, install it from [rustup.rs](https://rustup.rs/).

### Compilation

```bash
# Clone the repository
git clone https://github.com/xiaoxigua-1/deadbeef-plugin-discord-rpc.git
cd deadbeef-plugin-discord-rpc

# Build the plugin
cargo build --release
```

The compiled plugin will be located at `target/release/libdiscordrpc.so` (Linux) or `target/release/discordrpc.dll` (Windows).

## Installation

### Linux/macOS

```bash
# Run the install script
./scripts/install.sh
```

### Windows

```batch
# Run the install script
scripts\install.bat
```

After installation, restart DeaDBeeF and enable the plugin in the preferences. 

## Uninstallation

### Linux/macOS

```bash
# Run the uninstall script
./scripts/uninstall.sh
```

### Windows

```batch
# Run the uninstall script
scripts\uninstall.bat
```

## Configuration

The plugin can be configured through DeaDBeeF's preferences interface.  Available options include:

- Display format customization
- Album artwork settings
- MusicBrainz integration options

## Dependencies

- `discord-rich-presence` - Discord RPC client
- `lazy_static` & `once_cell` - For static initialization
- `json` - JSON parsing
- `urlencoding` - URL encoding utilities
- `bindgen` - FFI bindings generation (build-time)

## Project Structure

```
deadbeef-plugin-discord-rpc/
├── src/
│   ├── lib.rs           # Main plugin entry point
│   ├── discordrpc. rs    # Discord RPC client logic
│   ├── config. rs        # Configuration handling
│   ├── musicbrainz. rs   # MusicBrainz API integration
│   ├── util.rs          # Utility functions
│   ├── error.rs         # Error handling
│   └── deadbeef/        # DeaDBeeF FFI bindings
├── scripts/             # Installation and build scripts
│   ├── install.sh       # Linux/macOS installation
│   ├── install. bat      # Windows installation
│   ├── uninstall.sh     # Linux/macOS uninstallation
│   └── uninstall.bat    # Windows uninstallation
├── build.rs             # Build configuration
└── Cargo.toml           # Project manifest
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
