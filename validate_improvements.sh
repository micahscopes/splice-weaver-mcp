#!/bin/bash
# Validate the MCP improvements

echo "🔍 Validating MCP Resource Improvements"
echo "======================================="

# Test 1: Check if discovery resources are included
echo "1. Testing discovery resource availability..."
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/list"}' | \
cargo run --bin mcp-ast-grep 2>/dev/null | \
grep -q "🔍 Resource Discovery" && echo "✅ Discovery guide found" || echo "❌ Discovery guide missing"

# Test 2: Check if language listing is available
echo "2. Testing language listing availability..."
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/list"}' | \
cargo run --bin mcp-ast-grep 2>/dev/null | \
grep -q "📚 Available Languages" && echo "✅ Language listing found" || echo "❌ Language listing missing"

# Test 3: Check if catalog status is available
echo "3. Testing catalog status availability..."
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/list"}' | \
cargo run --bin mcp-ast-grep 2>/dev/null | \
grep -q "📊 Catalog Status" && echo "✅ Catalog status found" || echo "❌ Catalog status missing"

# Test 4: Check if popular language examples are directly accessible
echo "4. Testing direct language access..."
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/list"}' | \
cargo run --bin mcp-ast-grep 2>/dev/null | \
grep -q "JavaScript Examples" && echo "✅ JavaScript examples found" || echo "❌ JavaScript examples missing"

# Test 5: Check if catalog examples are loaded
echo "5. Testing catalog example loading..."
catalog_count=$(echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}
{"jsonrpc": "2.0", "id": 2, "method": "resources/list"}' | \
cargo run --bin mcp-ast-grep 2>/dev/null | \
grep -c "🔧\|📝" || echo "0")

if [ "$catalog_count" -gt 0 ]; then
    echo "✅ Found $catalog_count catalog example resources"
else
    echo "⚠️ No catalog examples found (check catalog status)"
fi

echo ""
echo "📊 Improvement Summary:"
echo "- Discovery guide for better navigation"
echo "- Language listing for easier language selection"
echo "- Catalog status for troubleshooting" 
echo "- Direct access to popular language examples"
echo "- Enhanced error handling instead of silent failures"
echo "- Improved resource names with emojis for better UX"
echo ""
echo "🎯 These changes make the MCP server more intuitive and friendly for smaller models!"