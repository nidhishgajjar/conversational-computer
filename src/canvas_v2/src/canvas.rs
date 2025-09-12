use skia_safe::{
    gpu::{self, gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    Color, ColorType, Paint, Point, Rect, Surface, Font, FontStyle, Typeface,
};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, Version},
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
use winit::window::Window;
use winit::event::{KeyEvent, ModifiersState};

use crate::input_field::InputField;
use crate::message::{Message, MessageRole};
use crate::animation::AnimationEngine;

pub struct Canvas {
    surface: Surface,
    gl_surface: glutin::surface::Surface<WindowSurface>,
    gl_context: glutin::context::PossiblyCurrentContext,
    
    // UI components
    input_field: InputField,
    messages: Vec<Message>,
    animations: AnimationEngine,
    
    // Window state
    width: i32,
    height: i32,
    scale_factor: f32,
    
    // Colors (dark theme)
    bg_color: Color,
    input_bg_color: Color,
    text_color: Color,
}

impl Canvas {
    pub fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        // Setup OpenGL context for Skia
        let (gl_context, gl_surface, fb_info) = Self::create_gl_context(window)?;
        
        // Create Skia surface
        let mut surface = Self::create_skia_surface(&gl_context, &fb_info, window)?;
        
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        
        Ok(Self {
            surface,
            gl_surface,
            gl_context,
            
            input_field: InputField::new(),
            messages: Vec::new(),
            animations: AnimationEngine::new(),
            
            width: size.width as i32,
            height: size.height as i32,
            scale_factor,
            
            // Dark theme colors
            bg_color: Color::from_rgb(25, 25, 35),
            input_bg_color: Color::from_rgb(35, 35, 45),
            text_color: Color::from_rgb(255, 255, 255),
        })
    }
    
    fn create_gl_context(window: &Window) -> Result<(
        glutin::context::PossiblyCurrentContext,
        glutin::surface::Surface<WindowSurface>,
        FramebufferInfo
    ), Box<dyn std::error::Error>> {
        // Create GL display
        let display_builder = DisplayBuilder::new()
            .with_window_builder(None);
            
        let (_, gl_display) = display_builder
            .build(&window.raw_window_handle(), Default::default(), |_| ())?;
        
        let gl_display = gl_display.ok_or("No GL display")?;
        
        // GL config
        let config_template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .build();
            
        let config = unsafe {
            gl_display
                .find_configs(config_template)?
                .next()
                .ok_or("No GL config")?
        };
        
        // Create GL context
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(None))
            .build(None);
            
        let gl_context = unsafe {
            gl_display.create_context(&config, &context_attributes)?
        };
        
        // Create surface
        let size = window.inner_size();
        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new()
            .build(
                window.raw_window_handle(),
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            );
            
        let gl_surface = unsafe {
            gl_display.create_window_surface(&config, &surface_attributes)?
        };
        
        let gl_context = gl_context.make_current(&gl_surface)?;
        
        // Load GL functions
        gl::load_with(|s| {
            gl_context.display().get_proc_address(s) as *const _
        });
        
        let fb_info = FramebufferInfo {
            fboid: 0,
            format: gpu::gl::Format::RGBA8.into(),
            protected: gpu::Protected::No,
        };
        
        Ok((gl_context, gl_surface, fb_info))
    }
    
    fn create_skia_surface(
        gl_context: &glutin::context::PossiblyCurrentContext,
        fb_info: &FramebufferInfo,
        window: &Window,
    ) -> Result<Surface, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let backend_render_target = BackendRenderTarget::new_gl(
            (size.width as i32, size.height as i32),
            0,
            8,
            *fb_info,
        );
        
        let context = gpu::direct_contexts::make_gl(gpu::gl::Interface::new_native(), None)
            .ok_or("Failed to create Skia context")?;
            
        Surface::from_backend_render_target(
            &context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        ).ok_or("Failed to create Skia surface".into())
    }
    
    pub fn render(&mut self) {
        let canvas = self.surface.canvas();
        canvas.clear(self.bg_color);
        
        // Draw messages (conversation history)
        self.draw_messages(canvas);
        
        // Draw input field (Spotlight-style at bottom center)
        self.draw_input_field(canvas);
        
        // Flush to GPU
        self.surface.flush_and_submit();
        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
    }
    
    fn draw_messages(&mut self, canvas: &mut skia_safe::Canvas) {
        let mut y = 100.0; // Start from top with padding
        
        let font = Font::from_typeface(
            Typeface::default(),
            16.0 * self.scale_factor,
        );
        
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        
        for message in &self.messages {
            // Animate message appearance
            let alpha = self.animations.get_message_alpha(&message.id);
            let y_offset = self.animations.get_message_y_offset(&message.id);
            
            // Role label
            paint.set_color(Color::from_rgba(150, 150, 170, alpha));
            let role_text = match message.role {
                MessageRole::User => "You",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };
            canvas.draw_str(role_text, Point::new(50.0, y + y_offset), &font, &paint);
            
            // Message content
            paint.set_color(Color::from_rgba(255, 255, 255, alpha));
            canvas.draw_str(&message.content, Point::new(50.0, y + y_offset + 25.0), &font, &paint);
            
            y += 60.0; // Spacing between messages
        }
    }
    
    fn draw_input_field(&mut self, canvas: &mut skia_safe::Canvas) {
        // Spotlight-style centered input
        let input_width = 600.0;
        let input_height = self.input_field.get_height();
        let x = (self.width as f32 - input_width) / 2.0;
        let y = self.height as f32 - input_height - 50.0;
        
        // Background with subtle shadow
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        
        // Shadow
        paint.set_color(Color::from_rgba(0, 0, 0, 50));
        let shadow_rect = Rect::from_xywh(x + 2.0, y + 2.0, input_width, input_height);
        canvas.draw_round_rect(shadow_rect, 8.0, 8.0, &paint);
        
        // Input background
        paint.set_color(self.input_bg_color);
        let rect = Rect::from_xywh(x, y, input_width, input_height);
        canvas.draw_round_rect(rect, 8.0, 8.0, &paint);
        
        // Draw input text
        let font = Font::from_typeface(
            Typeface::default(),
            14.0 * self.scale_factor,
        );
        
        paint.set_color(self.text_color);
        
        // Draw each line
        let lines = self.input_field.get_lines();
        let mut line_y = y + 25.0;
        
        for line in lines {
            canvas.draw_str(line, Point::new(x + 15.0, line_y), &font, &paint);
            line_y += 20.0;
        }
        
        // Draw cursor
        if self.animations.should_show_cursor() {
            let (cursor_x, cursor_y) = self.input_field.get_cursor_position();
            paint.set_color(Color::from_rgb(100, 200, 255));
            let cursor_rect = Rect::from_xywh(
                x + 15.0 + cursor_x as f32 * 8.0,
                y + 10.0 + cursor_y as f32 * 20.0,
                2.0,
                18.0,
            );
            canvas.draw_rect(cursor_rect, &paint);
        }
    }
    
    pub fn handle_keyboard(&mut self, event: KeyEvent) {
        use winit::keyboard::{KeyCode, PhysicalKey};
        
        if !event.state.is_pressed() {
            return;
        }
        
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Enter) => {
                if self.input_field.modifiers.shift() {
                    self.input_field.insert_newline();
                } else {
                    self.send_message();
                }
            }
            PhysicalKey::Code(KeyCode::Backspace) => {
                if self.input_field.modifiers.alt() {
                    self.input_field.delete_word_backward();
                } else {
                    self.input_field.backspace();
                }
            }
            PhysicalKey::Code(KeyCode::Delete) => {
                if self.input_field.modifiers.alt() {
                    self.input_field.delete_word_forward();
                } else {
                    self.input_field.delete();
                }
            }
            PhysicalKey::Code(KeyCode::ArrowLeft) => {
                if self.input_field.modifiers.super_key() {
                    self.input_field.move_to_line_start();
                } else if self.input_field.modifiers.alt() {
                    self.input_field.move_word_left();
                } else {
                    self.input_field.move_left();
                }
            }
            PhysicalKey::Code(KeyCode::ArrowRight) => {
                if self.input_field.modifiers.super_key() {
                    self.input_field.move_to_line_end();
                } else if self.input_field.modifiers.alt() {
                    self.input_field.move_word_right();
                } else {
                    self.input_field.move_right();
                }
            }
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.input_field.move_up();
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.input_field.move_down();
            }
            PhysicalKey::Code(KeyCode::KeyA) if self.input_field.modifiers.super_key() => {
                self.input_field.select_all();
            }
            _ => {
                // Handle character input
                if let Some(text) = &event.text {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            self.input_field.insert_char(ch);
                        }
                    }
                }
            }
        }
    }
    
    pub fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.input_field.modifiers = modifiers;
    }
    
    fn send_message(&mut self) {
        let text = self.input_field.get_text();
        if text.is_empty() {
            return;
        }
        
        // Add user message with animation
        let user_message = Message::new(MessageRole::User, text.clone());
        self.animations.add_message_animation(&user_message.id);
        self.messages.push(user_message);
        
        // Clear input
        self.input_field.clear();
        
        // Simulate AI response (will be real SPOC integration later)
        let ai_message = Message::new(
            MessageRole::Assistant,
            "I'll help you with that.".to_string(),
        );
        self.animations.add_message_animation(&ai_message.id);
        self.messages.push(ai_message);
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as i32;
        self.height = height as i32;
        // Recreate surface with new size
        // TODO: Implement surface recreation
    }
    
    pub fn update_animations(&mut self) -> bool {
        self.animations.update()
    }
}