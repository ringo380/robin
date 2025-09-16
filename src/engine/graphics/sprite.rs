use crate::engine::math::Vec2;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SpriteFrame {
    pub texture_name: String,
    pub uv_rect: UVRect,        // UV coordinates within texture
    pub pivot: Vec2,            // Pivot point (0.5, 0.5 = center)
    pub duration: f32,          // For animations (seconds)
}

#[derive(Clone, Debug)]
pub struct UVRect {
    pub x: f32,      // Left edge (0.0 - 1.0)
    pub y: f32,      // Top edge (0.0 - 1.0) 
    pub width: f32,  // Width (0.0 - 1.0)
    pub height: f32, // Height (0.0 - 1.0)
}

impl UVRect {
    pub fn full_texture() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }

    pub fn from_pixel_coords(x: u32, y: u32, width: u32, height: u32, texture_width: u32, texture_height: u32) -> Self {
        Self {
            x: x as f32 / texture_width as f32,
            y: y as f32 / texture_height as f32,
            width: width as f32 / texture_width as f32,
            height: height as f32 / texture_height as f32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpriteAnimation {
    pub name: String,
    pub frames: Vec<SpriteFrame>,
    pub looping: bool,
    pub total_duration: f32,
}

impl SpriteAnimation {
    pub fn new(name: String, looping: bool) -> Self {
        Self {
            name,
            frames: Vec::new(),
            looping,
            total_duration: 0.0,
        }
    }

    pub fn add_frame(&mut self, frame: SpriteFrame) {
        self.total_duration += frame.duration;
        self.frames.push(frame);
    }

    pub fn get_frame_at_time(&self, time: f32) -> Option<&SpriteFrame> {
        if self.frames.is_empty() {
            return None;
        }

        let mut normalized_time = if self.looping && self.total_duration > 0.0 {
            time % self.total_duration
        } else {
            time.min(self.total_duration)
        };

        let mut accumulated_time = 0.0;
        for frame in &self.frames {
            accumulated_time += frame.duration;
            if normalized_time <= accumulated_time {
                return Some(frame);
            }
        }

        // Return last frame if we've exceeded total duration (non-looping)
        self.frames.last()
    }
}

#[derive(Clone)]
pub struct Sprite {
    pub texture_name: String,
    pub uv_rect: UVRect,
    pub pivot: Vec2,
    pub size: Vec2,              // World space size
    pub color: [f32; 4],         // Tint color (RGBA)
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Sprite {
    pub fn new(texture_name: String) -> Self {
        Self {
            texture_name,
            uv_rect: UVRect::full_texture(),
            pivot: Vec2::new(0.5, 0.5), // Center pivot
            size: Vec2::new(32.0, 32.0), // Default size
            color: [1.0, 1.0, 1.0, 1.0], // White, fully opaque
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn with_uv_rect(mut self, uv_rect: UVRect) -> Self {
        self.uv_rect = uv_rect;
        self
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    pub fn with_pivot(mut self, x: f32, y: f32) -> Self {
        self.pivot = Vec2::new(x, y);
        self
    }

    pub fn flipped_x(mut self) -> Self {
        self.flip_x = true;
        self
    }

    pub fn flipped_y(mut self) -> Self {
        self.flip_y = true;
        self
    }

    pub fn get_uv_coords(&self) -> [f32; 8] {
        let mut left = self.uv_rect.x;
        let mut right = self.uv_rect.x + self.uv_rect.width;
        let mut top = self.uv_rect.y;
        let mut bottom = self.uv_rect.y + self.uv_rect.height;

        // Handle flipping
        if self.flip_x {
            std::mem::swap(&mut left, &mut right);
        }
        if self.flip_y {
            std::mem::swap(&mut top, &mut bottom);
        }

        // Return UV coordinates for quad vertices
        // [bottom-left, bottom-right, top-right, top-left]
        [
            left, bottom,   // Bottom-left
            right, bottom,  // Bottom-right  
            right, top,     // Top-right
            left, top,      // Top-left
        ]
    }
}

pub struct AnimatedSprite {
    pub sprite: Sprite,
    pub animations: HashMap<String, SpriteAnimation>,
    pub current_animation: Option<String>,
    pub animation_time: f32,
    pub playing: bool,
    pub speed_multiplier: f32,
}

impl AnimatedSprite {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            animations: HashMap::new(),
            current_animation: None,
            animation_time: 0.0,
            playing: false,
            speed_multiplier: 1.0,
        }
    }

    pub fn add_animation(&mut self, animation: SpriteAnimation) {
        let name = animation.name.clone();
        self.animations.insert(name, animation);
    }

    pub fn play_animation(&mut self, name: &str) {
        if self.animations.contains_key(name) {
            if self.current_animation.as_ref() != Some(&name.to_string()) {
                self.current_animation = Some(name.to_string());
                self.animation_time = 0.0;
            }
            self.playing = true;
        }
    }

    pub fn stop_animation(&mut self) {
        self.playing = false;
    }

    pub fn pause_animation(&mut self) {
        self.playing = false;
    }

    pub fn resume_animation(&mut self) {
        if self.current_animation.is_some() {
            self.playing = true;
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if !self.playing {
            return;
        }

        if let Some(animation_name) = &self.current_animation {
            if let Some(animation) = self.animations.get(animation_name) {
                self.animation_time += delta_time * self.speed_multiplier;

                // Check if animation completed (non-looping)
                if !animation.looping && self.animation_time >= animation.total_duration {
                    self.playing = false;
                    self.animation_time = animation.total_duration;
                }

                // Update sprite frame
                if let Some(frame) = animation.get_frame_at_time(self.animation_time) {
                    self.sprite.texture_name = frame.texture_name.clone();
                    self.sprite.uv_rect = frame.uv_rect.clone();
                    self.sprite.pivot = frame.pivot;
                }
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn is_animation_complete(&self) -> bool {
        if let Some(animation_name) = &self.current_animation {
            if let Some(animation) = self.animations.get(animation_name) {
                return !animation.looping && self.animation_time >= animation.total_duration;
            }
        }
        false
    }

    pub fn get_current_frame(&self) -> Option<&SpriteFrame> {
        if let Some(animation_name) = &self.current_animation {
            if let Some(animation) = self.animations.get(animation_name) {
                return animation.get_frame_at_time(self.animation_time);
            }
        }
        None
    }
}

pub struct SpriteManager {
    sprites: HashMap<String, Sprite>,
    animated_sprites: HashMap<String, AnimatedSprite>,
    sprite_atlases: HashMap<String, SpriteAtlas>,
}

impl SpriteManager {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            animated_sprites: HashMap::new(),
            sprite_atlases: HashMap::new(),
        }
    }

    pub fn create_sprite(&mut self, name: String, sprite: Sprite) {
        self.sprites.insert(name, sprite);
    }

    pub fn create_animated_sprite(&mut self, name: String, animated_sprite: AnimatedSprite) {
        self.animated_sprites.insert(name, animated_sprite);
    }

    pub fn get_sprite(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }

    pub fn get_sprite_mut(&mut self, name: &str) -> Option<&mut Sprite> {
        self.sprites.get_mut(name)
    }

    pub fn get_animated_sprite(&self, name: &str) -> Option<&AnimatedSprite> {
        self.animated_sprites.get(name)
    }

    pub fn get_animated_sprite_mut(&mut self, name: &str) -> Option<&mut AnimatedSprite> {
        self.animated_sprites.get_mut(name)
    }

    pub fn update_animated_sprites(&mut self, delta_time: f32) {
        for (_, animated_sprite) in self.animated_sprites.iter_mut() {
            animated_sprite.update(delta_time);
        }
    }

    pub fn add_sprite_atlas(&mut self, name: String, atlas: SpriteAtlas) {
        self.sprite_atlases.insert(name, atlas);
    }

    pub fn get_sprite_from_atlas(&self, atlas_name: &str, sprite_name: &str) -> Option<Sprite> {
        if let Some(atlas) = self.sprite_atlases.get(atlas_name) {
            return atlas.get_sprite(sprite_name);
        }
        None
    }

    // Convenience methods for common sprite operations
    pub fn create_simple_sprite(&mut self, name: String, texture_name: String, width: f32, height: f32) {
        let sprite = Sprite::new(texture_name).with_size(width, height);
        self.create_sprite(name, sprite);
    }

    pub fn create_ui_sprite(&mut self, name: String, texture_name: String, width: f32, height: f32) {
        let sprite = Sprite::new(texture_name)
            .with_size(width, height)
            .with_pivot(0.0, 0.0); // Top-left pivot for UI elements
        self.create_sprite(name, sprite);
    }
}

#[derive(Clone, Debug)]
pub struct SpriteAtlasFrame {
    pub name: String,
    pub uv_rect: UVRect,
    pub pivot: Vec2,
    pub size: Vec2, // Original pixel size
}

pub struct SpriteAtlas {
    pub texture_name: String,
    pub frames: HashMap<String, SpriteAtlasFrame>,
}

impl SpriteAtlas {
    pub fn new(texture_name: String) -> Self {
        Self {
            texture_name,
            frames: HashMap::new(),
        }
    }

    pub fn add_frame(&mut self, frame: SpriteAtlasFrame) {
        self.frames.insert(frame.name.clone(), frame);
    }

    pub fn get_sprite(&self, frame_name: &str) -> Option<Sprite> {
        if let Some(frame) = self.frames.get(frame_name) {
            Some(
                Sprite::new(self.texture_name.clone())
                    .with_uv_rect(frame.uv_rect.clone())
                    .with_pivot(frame.pivot.x, frame.pivot.y)
                    .with_size(frame.size.x, frame.size.y)
            )
        } else {
            None
        }
    }

    // Load from texture packer JSON format (common sprite atlas format)
    pub fn from_json(texture_name: String, json_data: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // This would parse JSON atlas data
        // For now, return empty atlas
        Ok(Self::new(texture_name))
    }

    // Create a simple grid-based atlas
    pub fn create_grid_atlas(
        texture_name: String,
        texture_width: u32,
        texture_height: u32,
        frame_width: u32,
        frame_height: u32,
        frame_count: u32,
    ) -> Self {
        let mut atlas = Self::new(texture_name);
        
        let cols = texture_width / frame_width;
        let rows = texture_height / frame_height;
        
        for i in 0..frame_count.min(cols * rows) {
            let col = i % cols;
            let row = i / cols;
            
            let frame = SpriteAtlasFrame {
                name: format!("frame_{}", i),
                uv_rect: UVRect::from_pixel_coords(
                    col * frame_width,
                    row * frame_height,
                    frame_width,
                    frame_height,
                    texture_width,
                    texture_height,
                ),
                pivot: Vec2::new(0.5, 0.5),
                size: Vec2::new(frame_width as f32, frame_height as f32),
            };
            
            atlas.add_frame(frame);
        }
        
        atlas
    }
}