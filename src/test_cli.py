#!/usr/bin/env python3
"""
SPOC CLI Test - Test the conversation flow without GUI
"""

class SPOCTest:
    def __init__(self):
        self.conversation = []
        
    def run(self):
        print("\n" + "="*50)
        print(" SPOC - Conversational Computer (CLI Test)")
        print("="*50)
        print("\nType your message and press Enter to send.")
        print("Type 'exit' to quit.\n")
        
        while True:
            # Show prompt (simulating bottom input bar)
            user_input = input("\n[SPOC Input] > ").strip()
            
            if user_input.lower() == 'exit':
                print("\nExiting SPOC...")
                break
                
            if not user_input:
                continue
                
            # Add to conversation
            self.conversation.append(("You", user_input))
            print(f"\n  You: {user_input}")
            
            # Simulate SPOC response
            response = f"I heard: '{user_input}'. SPOC conductor is learning..."
            self.conversation.append(("SPOC", response))
            print(f"  SPOC: {response}")
            
        print("\nConversation ended. Total exchanges:", len(self.conversation)//2)

if __name__ == "__main__":
    spoc = SPOCTest()
    spoc.run()