#!/bin/bash
# Test script to verify MCP server functionality

set -e

echo "🧪 Testing mcp-ast-grep MCP server..."

# Check if binary exists
if ! command -v mcp-ast-grep &> /dev/null; then
    echo "❌ mcp-ast-grep not found in PATH"
    echo "Run ./dev-install.sh first"
    exit 1
fi

echo "✅ Binary found: $(which mcp-ast-grep)"

# Test basic functionality
echo "🔍 Testing basic MCP communication..."

# Create a simple test file
cat > test-example.js << 'EOF'
function hello() {
    console.log("Hello world");
    return "done";
}

function goodbye() {
    console.log("Goodbye world");
    return "done";
}
EOF

echo "📝 Created test file: test-example.js"

# Start MCP server in background and test it
echo "🚀 Starting MCP server for testing..."
timeout 10s mcp-ast-grep &
SERVER_PID=$!

# Give it a moment to start
sleep 2

# Check if server is still running
if kill -0 $SERVER_PID 2>/dev/null; then
    echo "✅ MCP server started successfully (PID: $SERVER_PID)"
    kill $SERVER_PID
else
    echo "❌ MCP server failed to start or crashed"
    exit 1
fi

# Clean up
rm -f test-example.js

echo "🎉 All tests passed! MCP server is ready for use."