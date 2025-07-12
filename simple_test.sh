#!/bin/bash
# Simple test to show resource improvements

echo "🚀 Testing improved MCP resource access"
echo "========================================"

# Create test input
cat > test_input.json << 'EOF'
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/list"}
EOF

echo "📋 Listing resources..."
cargo run --bin mcp-ast-grep < test_input.json 2>/dev/null | grep -A 10 '"resources"' | head -20

echo ""
echo "✅ Resource listing test complete!"

# Cleanup
rm -f test_input.json