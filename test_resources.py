#!/usr/bin/env python3
"""
Simple test script to verify MCP resource functionality
"""

import json
import subprocess
import sys

def test_mcp_resources():
    """Test the MCP server's resource listing and reading capabilities"""
    
    # First, initialize the server
    init_request = {
        "jsonrpc": "2.0",
        "id": 0,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    }
    
    print("Initializing MCP server...")
    try:
        result = run_mcp_request(init_request)
        if "result" not in result:
            print("✗ Failed to initialize server")
            return False
        print("✓ Server initialized successfully")
    except Exception as e:
        print(f"✗ Error initializing server: {e}")
        return False
    
    # Test list_resources
    list_request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list",
        "params": {}
    }
    
    print("Testing resources/list...")
    try:
        result = run_mcp_request(list_request)
        if "result" in result and "resources" in result["result"]:
            resources = result["result"]["resources"]
            print(f"✓ Found {len(resources)} resources:")
            for resource in resources:
                print(f"  - {resource['name']}: {resource['uri']}")
        else:
            print("✗ No resources found or unexpected response format")
            return False
    except Exception as e:
        print(f"✗ Error testing resources/list: {e}")
        return False
    
    # Test reading binary-path resource
    read_request = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/read",
        "params": {
            "uri": "ast-grep://binary-path"
        }
    }
    
    print("\nTesting resources/read for binary-path...")
    try:
        result = run_mcp_request(read_request)
        if "result" in result and "contents" in result["result"]:
            content = result["result"]["contents"][0]["text"]
            print(f"✓ Binary path: {content}")
        else:
            print("✗ Failed to read binary-path resource")
            return False
    except Exception as e:
        print(f"✗ Error reading binary-path resource: {e}")
        return False
    
    # Test reading CLI reference resource
    read_request = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/read",
        "params": {
            "uri": "ast-grep://cli-reference"
        }
    }
    
    print("\nTesting resources/read for CLI reference...")
    try:
        result = run_mcp_request(read_request)
        if "result" in result and "contents" in result["result"]:
            content = result["result"]["contents"][0]["text"]
            print(f"✓ CLI reference contains {len(content)} characters")
            if "Binary Information" in content and "bundled" in content.lower():
                print("✓ CLI reference contains bundled binary information")
            else:
                print("✗ CLI reference missing expected bundled binary info")
                return False
        else:
            print("✗ Failed to read CLI reference resource")
            return False
    except Exception as e:
        print(f"✗ Error reading CLI reference resource: {e}")
        return False
    
    print("\n✓ All resource tests passed!")
    return True

def run_mcp_request(request):
    """Run a single MCP request and return the response"""
    try:
        process = subprocess.Popen(
            ["./target/release/mcp-ast-grep"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Send the request
        request_str = json.dumps(request)
        stdout, stderr = process.communicate(input=request_str, timeout=10)
        
        if stderr:
            print(f"stderr: {stderr}")
        
        # Parse the response
        for line in stdout.strip().split('\n'):
            if line:
                try:
                    response = json.loads(line)
                    if response.get("id") == request["id"]:
                        return response
                except json.JSONDecodeError:
                    continue
        
        raise Exception("No valid JSON response found")
        
    except subprocess.TimeoutExpired:
        process.kill()
        raise Exception("Request timed out")
    except Exception as e:
        raise Exception(f"Failed to run MCP request: {e}")

if __name__ == "__main__":
    success = test_mcp_resources()
    sys.exit(0 if success else 1)