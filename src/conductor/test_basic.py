#!/usr/bin/env python3
"""
Basic test of Claude Code SDK - Phase 1
Goal: Verify we can communicate with Claude Code
"""

import asyncio
from claude_code_sdk import query, ClaudeCodeOptions

async def test_basic():
    """Test basic 'hi' message"""
    print("Testing basic Claude Code SDK...")
    
    try:
        # Simple query without any tools
        response_text = ""
        async for message in query(
            prompt="Say hi in exactly 3 words",
            options=ClaudeCodeOptions(
                system_prompt="You are SPOC, a helpful conductor. Be very concise."
            )
        ):
            # Check what type of message we got
            if hasattr(message, 'result'):
                response_text = message.result
            elif hasattr(message, 'content'):
                response_text = str(message.content)
            else:
                print(f"Got message type: {type(message).__name__}")
                
        print(f"Response: {response_text}")
        return response_text
        
    except Exception as e:
        print(f"Error: {e}")
        return None

if __name__ == "__main__":
    result = asyncio.run(test_basic())
    if result:
        print("✓ Checkpoint 1: Basic communication works!")
    else:
        print("✗ Failed to get response")