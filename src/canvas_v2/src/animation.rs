use std::collections::HashMap;
use instant::Instant;

struct MessageAnimation {
    start_time: Instant,
    duration: f32,
}

pub struct AnimationEngine {
    message_animations: HashMap<String, MessageAnimation>,
    cursor_blink_start: Instant,
    cursor_visible: bool,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            message_animations: HashMap::new(),
            cursor_blink_start: Instant::now(),
            cursor_visible: true,
        }
    }
    
    pub fn add_message_animation(&mut self, message_id: &str) {
        self.message_animations.insert(
            message_id.to_string(),
            MessageAnimation {
                start_time: Instant::now(),
                duration: 0.3, // 300ms slide up animation
            },
        );
    }
    
    pub fn get_message_alpha(&self, message_id: &str) -> u8 {
        if let Some(anim) = self.message_animations.get(message_id) {
            let elapsed = anim.start_time.elapsed().as_secs_f32();
            let progress = (elapsed / anim.duration).min(1.0);
            
            // Ease out cubic
            let eased = 1.0 - (1.0 - progress).powi(3);
            (eased * 255.0) as u8
        } else {
            255
        }
    }
    
    pub fn get_message_y_offset(&self, message_id: &str) -> f32 {
        if let Some(anim) = self.message_animations.get(message_id) {
            let elapsed = anim.start_time.elapsed().as_secs_f32();
            let progress = (elapsed / anim.duration).min(1.0);
            
            // Ease out cubic
            let eased = 1.0 - (1.0 - progress).powi(3);
            20.0 * (1.0 - eased) // Start 20px below and animate up
        } else {
            0.0
        }
    }
    
    pub fn should_show_cursor(&self) -> bool {
        self.cursor_visible
    }
    
    pub fn update(&mut self) -> bool {
        // Update cursor blink
        let cursor_elapsed = self.cursor_blink_start.elapsed().as_millis();
        if cursor_elapsed > 500 {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_start = Instant::now();
        }
        
        // Clean up finished animations
        let finished: Vec<String> = self.message_animations
            .iter()
            .filter(|(_, anim)| anim.start_time.elapsed().as_secs_f32() > anim.duration + 0.1)
            .map(|(id, _)| id.clone())
            .collect();
            
        for id in finished {
            self.message_animations.remove(&id);
        }
        
        // Return true if we need to redraw
        !self.message_animations.is_empty() || cursor_elapsed < 550
    }
}