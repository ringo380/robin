use crate::engine::math::{Vec2, Vec3, Mat4};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub mod serialization;
pub mod manager;

pub use serialization::*;
pub use manager::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.position);
        let rotation = Mat4::from_angle_z(cgmath::Rad(self.rotation));
        let scale = Mat4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0);
        
        translation * rotation * scale
    }
}

pub struct GameObject {
    pub id: u32,
    pub transform: Transform,
    pub active: bool,
    pub components: HashMap<String, Box<dyn std::any::Any>>,
}

impl GameObject {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            transform: Transform::new(),
            active: true,
            components: HashMap::new(),
        }
    }

    pub fn add_component<T: 'static>(&mut self, name: &str, component: T) {
        self.components.insert(name.to_string(), Box::new(component));
    }

    pub fn get_component<T: 'static>(&self, name: &str) -> Option<&T> {
        self.components
            .get(name)
            .and_then(|component| component.downcast_ref::<T>())
    }
}

pub struct Scene {
    pub name: String,
    objects: HashMap<u32, GameObject>,
    next_id: u32,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            name: "Scene".to_string(),
            objects: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_object(&mut self) -> &mut GameObject {
        let id = self.next_id;
        self.next_id += 1;
        
        let object = GameObject::new(id);
        self.objects.insert(id, object);
        self.objects.get_mut(&id).unwrap()
    }

    pub fn get_object(&self, id: u32) -> Option<&GameObject> {
        self.objects.get(&id)
    }

    pub fn get_object_mut(&mut self, id: u32) -> Option<&mut GameObject> {
        self.objects.get_mut(&id)
    }

    pub fn remove_object(&mut self, id: u32) -> Option<GameObject> {
        self.objects.remove(&id)
    }

    pub fn objects(&self) -> impl Iterator<Item = &GameObject> {
        self.objects.values()
    }

    pub fn objects_mut(&mut self) -> impl Iterator<Item = &mut GameObject> {
        self.objects.values_mut()
    }

    pub fn to_serializable(&self) -> serialization::SerializableScene {
        serialization::SerializableScene::default()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn load_from_serializable(&mut self, _serializable: &serialization::SerializableScene) {
        // Load scene from serializable representation
    }
}