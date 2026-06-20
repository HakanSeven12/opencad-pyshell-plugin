# PYSHELL Plugin

> **Status:** Phase 1 Implementation (Core Infrastructure)

Interactive Python shell plugin for OpenCADStudio providing scriptable access to CAD documents.

## Features (Planned)

- [x] Interactive Python REPL (Phase 1)
- [ ] Document entity access (points, lines, circles, etc.) (Phase 2)
- [ ] Entity creation (Phase 2)
- [ ] Layer management (Phase 2)
- [ ] XDATA read/write (Phase 4)
- [ ] Undo/redo support (Phase 4)

## Build

```bash
# Navigate to plugin directory
cd crates/pyshell-plugin

# Build in release mode
cargo build --release
```

## Installation

Copy the built plugin to OpenCADStudio's plugin directory:

### Windows
```powershell
# Create plugin directory
mkdir %APPDATA%\OpenCADStudio\plugins\opencad.pyshell

# Copy plugin files
copy target\release\pyshell_plugin.dll %APPDATA%\OpenCADStudio\plugins\opencad.pyshell\
copy plugin.toml %APPDATA%\OpenCADStudio\plugins\opencad.pyshell\
```

### macOS
```bash
# Create plugin directory
mkdir -p ~/Library/Application\ Support/OpenCADStudio/plugins/opencad.pyshell

# Copy plugin files
cp target/release/libpyshell_plugin.dylib ~/Library/Application\ Support/OpenCADStudio/plugins/opencad.pyshell/
cp plugin.toml ~/Library/Application\ Support/OpenCADStudio/plugins/opencad.pyshell/
```

### Linux
```bash
# Install Python development headers (required for PyO3)
sudo apt-get install python3-dev

# Create plugin directory
mkdir -p ~/.config/OpenCADStudio/plugins/opencad.pyshell

# Copy plugin files
cp target/release/libpyshell_plugin.so ~/.config/OpenCADStudio/plugins/opencad.pyshell/
cp plugin.toml ~/.config/OpenCADStudio/plugins/opencad.pyshell/
```

## Usage

1. Launch OpenCADStudio
2. Open or create a CAD document
3. Click the "Python Shell" button in the ribbon (under "Python" tab)
4. The Python REPL will launch in the console
5. Enter Python commands:
   ```python
   print("Hello from Python!")
   x = 42
   x * 2
   [i**2 for i in range(10)]
   exit()  # or quit() to exit
   ```

## Requirements

- Rust 1.70+
- Python 3.11+ (standard installation from python.org)
  - Windows/macOS: Standard installer is sufficient
  - Linux: Install `python3-dev` package
- OpenCADStudio (with matching acadrust version 0.3.4)

## Project Structure

```
pyshell-plugin/
├── Cargo.toml          # Plugin manifest and dependencies
├── plugin.toml         # Plugin metadata for OpenCADStudio
├── README.md           # This file
└── src/
    └── lib.rs          # Plugin entry point and implementation
```

## Architecture

This plugin uses:
- **PyO3 0.21** with `extension-module` feature for Python bindings
- **No libpython linking** - works with standard Python installations
- **HostApi** from `ocs_plugin_api` for document access
- **acadrust 0.3.4** for CAD document model (version-matched to host)

## Troubleshooting

### Plugin doesn't load
- Verify `plugin.toml` matches the `id` in `src/lib.rs`
- Check file names: `pyshell_plugin.dll` (Windows), `libpyshell_plugin.so` (Linux), `libpyshell_plugin.dylib` (macOS)
- Check OpenCADStudio log for error messages

### Python REPL doesn't start
- Verify Python 3.11+ is installed and in PATH
- On Linux: Ensure `python3-dev` is installed (`sudo apt-get install python3-dev`)
- Check that PyO3 version is compatible with your Python

### Build errors
- **Linker errors on Linux:** Install `python3-dev`
- **Cannot find Python.h:** Install Python development headers
- **Version mismatches:** Ensure acadrust version matches OpenCADStudio host

## License

GPL-3.0-only

## Contributing

See [PHASE1_PYSHELL_IMPLEMENTATION.md](../PHASE1_PYSHELL_IMPLEMENTATION.md) for implementation details.

See [option1_bplus.md](../../option1_bplus.md) for the full 5-phase implementation plan.
