/*!
 * Color utilities for Robin Engine
 */

use serde::{Deserialize, Serialize};

/// RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self { r: arr[0], g: arr[1], b: arr[2], a: arr[3] }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<wgpu::Color> for Color {
    fn from(color: wgpu::Color) -> Self {
        Self { 
            r: color.r as f32, 
            g: color.g as f32, 
            b: color.b as f32, 
            a: color.a as f32 
        }
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        wgpu::Color { 
            r: color.r as f64, 
            g: color.g as f64, 
            b: color.b as f64, 
            a: color.a as f64 
        }
    }
}