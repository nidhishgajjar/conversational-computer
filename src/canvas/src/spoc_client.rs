//! SPOC Client - Communicates with Python conductor via Unix socket

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use serde::{Deserialize, Serialize};

const SOCKET_PATH: &str = "/tmp/spoc.sock";

/// Response block from SPOC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SPOCBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub content: Option<String>,
    pub command: Option<String>,
    pub output: Option<String>,
    pub role: String,
}

/// SPOC client for IPC communication
pub struct SPOCClient;

impl SPOCClient {
    /// Query SPOC and get response blocks
    pub fn query(input: &str) -> Result<Vec<SPOCBlock>, String> {
        // Try to connect to socket
        let mut stream = UnixStream::connect(SOCKET_PATH)
            .map_err(|e| format!("Failed to connect to SPOC: {}. Is the server running?", e))?;
        
        // Send input
        stream.write_all(input.as_bytes())
            .map_err(|e| format!("Failed to send to SPOC: {}", e))?;
        
        // Read response
        let mut response = Vec::new();
        stream.read_to_end(&mut response)
            .map_err(|e| format!("Failed to read from SPOC: {}", e))?;
        
        // Parse JSON response
        let response_str = String::from_utf8(response)
            .map_err(|e| format!("Invalid UTF-8 from SPOC: {}", e))?;
        
        // Debug: print what we got
        eprintln!("SPOC response: {}", response_str);
        
        let blocks: Vec<SPOCBlock> = serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse SPOC response: {}. Raw: {}", e, response_str))?;
        
        eprintln!("Parsed {} blocks", blocks.len());
        
        Ok(blocks)
    }
    
    /// Check if SPOC server is available
    pub fn is_available() -> bool {
        UnixStream::connect(SOCKET_PATH).is_ok()
    }
}