#!/usr/bin/env python3
"""
Mock SPOC Conductor - Phase 1
Simple test conductor that returns hardcoded responses
This will be replaced with real AI later
"""

import json
import subprocess
import datetime

class MockConductor:
    """Mock conductor for testing IPC and blocks"""
    
    def __init__(self):
        self.history = []
    
    def process(self, user_input: str) -> list:
        """Process input and return structured blocks"""
        
        # Store in history
        self.history.append({"role": "user", "content": user_input})
        
        # Lowercase for matching
        input_lower = user_input.lower()
        
        # Basic pattern matching for testing
        blocks = []
        
        if "hi" in input_lower or "hello" in input_lower:
            blocks.append({
                "type": "text",
                "content": "Hello! I'm SPOC, your conversational computer conductor.",
                "role": "assistant"
            })
            
        elif "time" in input_lower:
            now = datetime.datetime.now()
            blocks.append({
                "type": "text", 
                "content": f"Current time: {now.strftime('%Y-%m-%d %H:%M:%S')}",
                "role": "assistant"
            })
            
        elif "list files" in input_lower or "ls" in input_lower:
            # Execute command and return terminal block
            try:
                result = subprocess.run(['ls', '-la'], 
                                      capture_output=True, 
                                      text=True,
                                      timeout=5)
                blocks.append({
                    "type": "terminal",
                    "command": "ls -la",
                    "output": result.stdout,
                    "role": "assistant"
                })
            except Exception as e:
                blocks.append({
                    "type": "text",
                    "content": f"Error running command: {e}",
                    "role": "assistant"
                })
                
        elif "pwd" in input_lower or "where am i" in input_lower:
            try:
                result = subprocess.run(['pwd'], 
                                      capture_output=True,
                                      text=True,
                                      timeout=5)
                blocks.append({
                    "type": "terminal",
                    "command": "pwd",
                    "output": result.stdout.strip(),
                    "role": "assistant"
                })
            except Exception as e:
                blocks.append({
                    "type": "text",
                    "content": f"Error: {e}",
                    "role": "assistant"
                })
                
        elif "clear" in input_lower:
            self.history.clear()
            blocks.append({
                "type": "system",
                "content": "Conversation cleared",
                "role": "system"
            })
            
        else:
            # Default echo response
            blocks.append({
                "type": "text",
                "content": f"You said: '{user_input}'. I'm a mock conductor for testing.",
                "role": "assistant"
            })
        
        # Store assistant response
        for block in blocks:
            if block.get("role") == "assistant":
                self.history.append({"role": "assistant", "content": block})
        
        return blocks


def test_conductor():
    """Test the mock conductor"""
    conductor = MockConductor()
    
    test_inputs = [
        "hi",
        "what time is it?",
        "list files",
        "where am i?",
        "random text"
    ]
    
    for inp in test_inputs:
        print(f"\n> {inp}")
        response = conductor.process(inp)
        print(json.dumps(response, indent=2))


if __name__ == "__main__":
    print("Testing Mock Conductor...")
    test_conductor()
    print("\n✓ Mock conductor working!")