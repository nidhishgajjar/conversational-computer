#!/usr/bin/env python3
"""
SPOC Conductor - Claude Code SDK Implementation
Uses Claude Code SDK with existing authentication
"""

import json
import asyncio
import os
import sys
from typing import List, Dict
from claude_code_sdk import query, ClaudeCodeOptions

# Ensure PATH includes Claude CLI
os.environ['PATH'] = f"{os.path.expanduser('~/.local/bin')}:{os.environ.get('PATH', '')}"

class Conductor:
    """SPOC Conductor using Claude Code SDK"""
    
    def __init__(self):
        self.history = []
        self.options = ClaudeCodeOptions(
            system_prompt="""You are SPOC, the conductor of a conversational computer.
Be concise - keep responses to 1-2 sentences maximum.
When asked to run commands or list files, use the Bash tool.
You help users interact with their computer through natural conversation.""",
            allowed_tools=["Bash"],
            permission_mode="bypassPermissions",
            max_turns=1,
            cwd=os.getcwd()  # Use current working directory
        )
        print("Conductor: Using Claude Code SDK")
    
    def process(self, user_input: str) -> List[Dict]:
        """Process user input using Claude Code SDK"""
        
        # Add to history
        self.history.append({"role": "user", "content": user_input})
        
        # Use Claude Code SDK with proper async handling
        try:
            # Create a new event loop for this sync context
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            
            # Run the async function with timeout
            result = loop.run_until_complete(
                asyncio.wait_for(
                    self._process_with_sdk(user_input),
                    timeout=30.0
                )
            )
            
            loop.close()
            return result
            
        except asyncio.TimeoutError:
            print("SDK timeout")
            return [{
                "type": "text",
                "content": "Request timed out. Please try again.",
                "role": "system"
            }]
        except Exception as e:
            print(f"SDK Error: {e}")
            return [{
                "type": "text",
                "content": f"Error: {str(e)[:100]}",
                "role": "system"
            }]
    
    async def _process_with_sdk(self, user_input: str) -> List[Dict]:
        """Process with Claude Code SDK"""
        blocks = []
        
        # Collect all messages
        all_messages = []
        tool_uses = {}
        
        async for message in query(prompt=user_input, options=self.options):
            all_messages.append(message)
            msg_type = type(message).__name__
            
            # Process message immediately
            if msg_type == "ResultMessage":
                # Final result - this is what we want
                if hasattr(message, 'result') and message.result:
                    # Only add ResultMessage if we don't have tool results
                    if not blocks and message.result:
                        blocks.append({
                            "type": "text",
                            "content": message.result,
                            "role": "assistant"
                        })
            
            elif msg_type == "AssistantMessage" and hasattr(message, 'content'):
                # Check for tool use
                for block in message.content:
                    block_type = type(block).__name__
                    if block_type == "ToolUseBlock":
                        if hasattr(block, 'id') and hasattr(block, 'name'):
                            tool_info = {
                                'name': block.name,
                                'input': {}
                            }
                            
                            # Try to get input
                            if hasattr(block, 'input'):
                                tool_info['input'] = block.input
                                if block.name == "Bash" and isinstance(block.input, dict):
                                    tool_info['command'] = block.input.get('command', 'Command')
                            
                            tool_uses[block.id] = tool_info
            
            elif msg_type == "UserMessage" and hasattr(message, 'content'):
                # Tool results come in UserMessage!
                for block in message.content:
                    block_type = type(block).__name__
                    if block_type == "ToolResultBlock":
                        tool_id = getattr(block, 'tool_use_id', None)
                        content = getattr(block, 'content', '')
                        
                        if tool_id and tool_id in tool_uses:
                            tool_info = tool_uses[tool_id]
                            if tool_info['name'] == "Bash" and content:
                                # Terminal block with command output
                                blocks.append({
                                    "type": "terminal",
                                    "command": tool_info.get('command', 'Command executed'),
                                    "output": content,
                                    "role": "assistant"
                                })
        
        # Ensure we return something
        if not blocks:
            blocks.append({
                "type": "text",
                "content": "I can help you with system tasks. Try asking me to list files or run commands.",
                "role": "assistant"
            })
        
        return blocks


def test_conductor():
    """Test the conductor"""
    print("\n" + "="*50)
    print("SPOC Conductor - Claude Code SDK")
    print("="*50)
    
    conductor = Conductor()
    print("-"*50)
    
    test_inputs = [
        "Hello!",
        "What is 2+2?",
        "List the files",
        "Show current time"
    ]
    
    for inp in test_inputs:
        print(f"\n> {inp}")
        print("Processing...")
        
        response = conductor.process(inp)
        
        for block in response:
            if block['type'] == 'terminal':
                print(f"[Terminal: {block['command']}]")
                output = block['output']
                if len(output) > 200:
                    output = output[:200] + "..."
                print(output)
            else:
                print(f"[{block['role'].title()}]: {block['content']}")
        
        print("-"*30)


if __name__ == "__main__":
    test_conductor()