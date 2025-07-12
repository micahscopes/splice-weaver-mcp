#!/usr/bin/env python3
"""
Simple test script to demonstrate the improved MCP resource access
"""

import subprocess
import json
import sys

def run_mcp_test():
    """Run a simple MCP test to list and read resources"""
    
    # Start the MCP server
    server_cmd = ["cargo", "run", "--bin", "mcp-ast-grep"]
    
    # Create a simple test client interaction
    test_requests = [
        # Initialize
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        },
        # List resources
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/list"
        },
        # Read discovery guide
        {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/read",
            "params": {"uri": "ast-grep://discover"}
        },
        # Read languages list
        {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/read",
            "params": {"uri": "ast-grep://languages"}
        },
        # Check catalog status
        {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {"uri": "ast-grep://catalog-status"}
        }
    ]
    
    print("🚀 Starting MCP Server Test")
    print("=" * 50)
    
    try:
        proc = subprocess.Popen(
            server_cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Send requests
        for req in test_requests:
            json_req = json.dumps(req) + "\n"
            proc.stdin.write(json_req)
            proc.stdin.flush()
        
        # Close stdin to signal end of input
        proc.stdin.close()
        
        # Read responses
        responses = []
        for line in proc.stdout:
            try:
                if line.strip():
                    response = json.loads(line.strip())
                    responses.append(response)
            except json.JSONDecodeError:
                continue
        
        # Wait for process to complete
        proc.wait(timeout=10)
        
        # Analyze responses
        print("📊 Resource Listing Results")
        print("-" * 30)
        
        for i, response in enumerate(responses):
            if "result" in response:
                result = response["result"]
                
                if i == 1:  # resources/list response
                    resources = result.get("resources", [])
                    print(f"✅ Found {len(resources)} resources")
                    
                    # Show first few resources to demonstrate improvements
                    discovery_resources = [r for r in resources if "🔍" in r.get("name", "") or "📚" in r.get("name", "")]
                    catalog_resources = [r for r in resources if "🔧" in r.get("name", "") or "📝" in r.get("name", "")]
                    
                    print(f"📋 Discovery Resources: {len(discovery_resources)}")
                    for r in discovery_resources[:3]:
                        print(f"   • {r['name']}")
                    
                    print(f"📚 Catalog Resources: {len(catalog_resources)}")
                    for r in catalog_resources[:3]:
                        print(f"   • {r['name']}")
                
                elif i == 2:  # discovery guide
                    content = result.get("contents", [{}])[0].get("text", "")
                    lines = content.split('\n')
                    print(f"📖 Discovery Guide: {len(lines)} lines")
                    # Show first few lines
                    for line in lines[:5]:
                        if line.strip():
                            print(f"   {line[:70]}...")
                            break
                
                elif i == 3:  # languages list
                    content = result.get("contents", [{}])[0].get("text", "")
                    js_mentioned = "JavaScript" in content
                    python_mentioned = "Python" in content
                    print(f"🌐 Languages Guide: JS={js_mentioned}, Python={python_mentioned}")
                
                elif i == 4:  # catalog status
                    content = result.get("contents", [{}])[0].get("text", "")
                    if "✅ LOADED" in content:
                        print("📊 Catalog Status: ✅ LOADED")
                    elif "❌ FAILED" in content:
                        print("📊 Catalog Status: ❌ FAILED")
                    else:
                        print("📊 Catalog Status: Unknown")
        
        print("\n🎉 MCP Resource Test Complete!")
        return True
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        return False

if __name__ == "__main__":
    success = run_mcp_test()
    sys.exit(0 if success else 1)