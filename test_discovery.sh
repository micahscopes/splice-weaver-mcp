#!/bin/bash
# Test the discovery functionality

echo "🔧 Testing Discovery Resource"
echo "============================="

# Create a more comprehensive test
(cat << 'EOF'
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/read", "params": {"uri": "ast-grep://discover"}}
EOF
) | cargo run --bin mcp-ast-grep 2>/dev/null | grep -A 5 "Resource Discovery"

echo ""
echo "🌐 Testing Language Resource"
echo "============================"

(cat << 'EOF'
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/read", "params": {"uri": "ast-grep://languages"}}
EOF
) | cargo run --bin mcp-ast-grep 2>/dev/null | grep -A 5 "JavaScript\|Python\|Rust"

echo ""
echo "✅ Discovery tests complete!"