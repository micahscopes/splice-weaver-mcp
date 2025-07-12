#!/bin/bash
# Development installation script for mcp-ast-grep

set -e

echo "🔧 Setting up dev environment for mcp-ast-grep..."

# Build the project
echo "📦 Building project..."
cargo build --release

# Create a symlink in a directory that's in PATH (usually ~/.cargo/bin)
CARGO_BIN="$HOME/.cargo/bin"
BINARY_PATH="$(pwd)/target/release/mcp-ast-grep"

if [ ! -d "$CARGO_BIN" ]; then
    echo "📁 Creating ~/.cargo/bin directory..."
    mkdir -p "$CARGO_BIN"
fi

echo "🔗 Creating symlink in $CARGO_BIN..."
ln -sf "$BINARY_PATH" "$CARGO_BIN/mcp-ast-grep"

echo "✅ Dev installation complete!"
echo ""
echo "📋 Next steps:"
echo "  1. Add ~/.cargo/bin to your PATH if not already added:"
echo "     export PATH=\"\$HOME/.cargo/bin:\$PATH\""
echo ""
echo "  2. Test the installation:"
echo "     mcp-ast-grep --help"
echo ""
echo "  3. Add to your MCP client config:"
echo "     {\"mcp\": {\"servers\": {\"ast-grep\": {\"command\": \"mcp-ast-grep\", \"args\": []}}}}"
echo ""
echo "🔄 To update after making changes:"
echo "   cargo build --release && ./dev-install.sh"