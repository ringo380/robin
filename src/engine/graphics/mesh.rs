/*!
 * Mesh utilities for Robin Engine
 */

use crate::engine::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// Vertex data structure
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2, color: [f32; 4]) -> Self {
        Self { position, normal, uv, color }
    }
}

/// Mesh data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub name: String,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, name: String) -> Self {
        Self { vertices, indices, name }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            name: "default".to_string(),
        }
    }
}