use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: SystemTime::now(),
        }
    }
}

// Add uuid to Cargo.toml dependencies