#!/usr/bin/env python3
"""
SPOC Server - Final Implementation
Unix socket server for Canvas IPC
"""

import socket
import os
import json
import threading
from conductor import Conductor

SOCKET_PATH = "/tmp/spoc.sock"

class SPOCServer:
    def __init__(self):
        self.conductor = Conductor()
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
        print("-" * 40)
        
        try:
            while True:
                client, addr = server.accept()
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
            # Read message
            data = client.recv(4096)
            if not data:
                return
            
            message = data.decode('utf-8').strip()
            print(f"Received: {message}")
            
            # Process with conductor
            response_blocks = self.conductor.process(message)
            
            # Send response
            response = json.dumps(response_blocks)
            client.send(response.encode('utf-8'))
            
            print(f"Sent: {response_blocks[0].get('type')} response")
            
        except Exception as e:
            print(f"Error: {e}")
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


if __name__ == "__main__":
    server = SPOCServer()
    server.start()