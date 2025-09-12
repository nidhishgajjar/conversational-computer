#!/usr/bin/env python3
"""
SPOC IPC Server
Provides Unix socket interface for Canvas to communicate with conductor
"""

import socket
import os
import json
import threading
from mock_conductor import MockConductor

SOCKET_PATH = "/tmp/spoc.sock"

class SPOCServer:
    def __init__(self):
        self.conductor = MockConductor()
        self.socket_path = SOCKET_PATH
        
    def start(self):
        """Start the Unix socket server"""
        # Remove old socket if exists
        if os.path.exists(self.socket_path):
            os.remove(self.socket_path)
        
        # Create Unix socket
        server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        server.bind(self.socket_path)
        server.listen(5)
        
        print(f"SPOC Server listening on {self.socket_path}")
        
        try:
            while True:
                client, addr = server.accept()
                # Handle each client in a thread
                thread = threading.Thread(target=self.handle_client, args=(client,))
                thread.daemon = True
                thread.start()
        except KeyboardInterrupt:
            print("\nShutting down server...")
        finally:
            server.close()
            if os.path.exists(self.socket_path):
                os.remove(self.socket_path)
    
    def handle_client(self, client):
        """Handle a client connection"""
        try:
            # Read data (max 4KB)
            data = client.recv(4096)
            if not data:
                return
            
            # Decode message
            message = data.decode('utf-8').strip()
            print(f"Received: {message}")
            
            # Process with conductor
            response_blocks = self.conductor.process(message)
            
            # Send response as JSON
            response = json.dumps(response_blocks)
            client.send(response.encode('utf-8'))
            
            print(f"Sent: {len(response)} bytes")
            
        except Exception as e:
            print(f"Error handling client: {e}")
            error_response = json.dumps([{
                "type": "error",
                "content": str(e),
                "role": "system"
            }])
            try:
                client.send(error_response.encode('utf-8'))
            except:
                pass
        finally:
            client.close()


def test_client():
    """Test client to verify server works"""
    import time
    
    # Give server time to start
    time.sleep(1)
    
    test_messages = ["hi", "what time is it?", "list files"]
    
    for msg in test_messages:
        try:
            # Connect to server
            client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            client.connect(SOCKET_PATH)
            
            # Send message
            print(f"\nClient sending: {msg}")
            client.send(msg.encode('utf-8'))
            
            # Receive response
            response = client.recv(4096).decode('utf-8')
            blocks = json.loads(response)
            
            print(f"Client received: {json.dumps(blocks, indent=2)}")
            
            client.close()
            
        except Exception as e:
            print(f"Client error: {e}")
            break
        
        time.sleep(0.5)


if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "test":
        # Run test client
        print("Running test client...")
        test_client()
    else:
        # Run server
        server = SPOCServer()
        server.start()