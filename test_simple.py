#!/usr/bin/env python3
"""
Simple test to verify the MCP server binary bundling works
"""

import json
import subprocess
import sys

def test_server():
    """Test basic server functionality"""
    try:
        # Start the server process
        process = subprocess.Popen(
            ["./target/release/mcp-ast-grep"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Send initialization request
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}, "resources": {}},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }
        
        print("Sending initialization request...")
        request_str = json.dumps(init_request) + "\n"
        
        # Send the request and immediately close stdin to signal end
        stdout, stderr = process.communicate(input=request_str, timeout=5)
        
        print(f"Server output: {stdout}")
        if stderr:
            print(f"Server errors: {stderr}")
        
        # Check if we got a valid response
        if "protocolVersion" in stdout and "capabilities" in stdout:
            print("✓ Server responds to initialization")
            return True
        else:
            print("✗ Server did not respond correctly")
            return False
            
    except subprocess.TimeoutExpired:
        process.kill()
        print("✗ Server timed out")
        return False
    except Exception as e:
        print(f"✗ Error: {e}")
        return False

if __name__ == "__main__":
    print("Testing MCP ast-grep server with bundled binary...")
    success = test_server()
    
    # Also test if the binary manager can find or download the binary
    print("\nTesting binary manager functionality...")
    try:
        # Try to run a simple ast-grep command to see if it works
        from src.binary_manager import BinaryManager
        import asyncio
        
        async def test_binary():
            manager = BinaryManager()
            binary_path = await manager.ensure_binary()
            print(f"✓ Binary available at: {binary_path}")
            return True
        
        binary_success = asyncio.run(test_binary())
        success = success and binary_success
        
    except Exception as e:
        print(f"Note: Binary manager test failed (expected if ast-grep not installed): {e}")
    
    print(f"\nOverall result: {'✓ SUCCESS' if success else '✗ FAILED'}")
    sys.exit(0 if success else 1)