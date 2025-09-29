// Dynamic Texture Atlas System for efficient GPU texture management
// Automatically packs multiple textures into atlases to minimize state changes

use std::collections::HashMap;
use metal::*;
use crate::material_batching::MaterialType;

/// Maximum size for texture atlases (2048x2048 for optimal GPU memory usage)
pub const MAX_ATLAS_SIZE: u32 = 2048;

/// Minimum texture resolution for atlas packing
pub const MIN_TEXTURE_SIZE: u32 = 16;

/// Standard texture sizes supported by the atlas system
pub const STANDARD_TEXTURE_SIZES: [u32; 6] = [16, 32, 64, 128, 256, 512];

/// UV coordinates for a texture region in an atlas
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtlasUV {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

impl AtlasUV {
    pub fn new(u_min: f32, v_min: f32, u_max: f32, v_max: f32) -> Self {
        Self { u_min, v_min, u_max, v_max }
    }

    /// Get UV coordinates for quad vertices [bottom-left, bottom-right, top-right, top-left]
    pub fn to_quad_coords(&self) -> [[f32; 2]; 4] {
        [
            [self.u_min, self.v_min], // bottom-left
            [self.u_max, self.v_min], // bottom-right
            [self.u_max, self.v_max], // top-right
            [self.u_min, self.v_max], // top-left
        ]
    }
}

/// Represents a texture loaded into the atlas system
#[derive(Debug, Clone)]
pub struct AtlasTexture {
    pub id: u32,
    pub material_type: MaterialType,
    pub size: u32,
    pub atlas_id: u32,
    pub uv_coords: AtlasUV,
    pub is_emissive: bool,
    pub is_transparent: bool,
}

/// Rectangle packing node for efficient texture layout
#[derive(Debug, Clone)]
struct PackingNode {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    used: bool,
    right: Option<Box<PackingNode>>,
    down: Option<Box<PackingNode>>,
}

impl PackingNode {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x, y, width, height,
            used: false,
            right: None,
            down: None,
        }
    }

    /// Check if this node can fit a texture of given dimensions
    fn can_fit(&self, width: u32, height: u32) -> bool {
        if self.used {
            // Check child nodes
            let right_fits = self.right.as_ref()
                .map(|r| r.can_fit(width, height))
                .unwrap_or(false);
            let down_fits = self.down.as_ref()
                .map(|d| d.can_fit(width, height))
                .unwrap_or(false);
            right_fits || down_fits
        } else {
            width <= self.width && height <= self.height
        }
    }

    /// Find and mark a spot for a texture, returns position
    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if self.used {
            // Try the right side first, then down
            if let Some(ref mut right) = self.right {
                if let Some(pos) = right.allocate(width, height) {
                    return Some(pos);
                }
            }
            if let Some(ref mut down) = self.down {
                return down.allocate(width, height);
            }
            None
        } else if width <= self.width && height <= self.height {
            self.split(width, height);
            Some((self.x, self.y))
        } else {
            None
        }
    }

    /// Split this node to accommodate a texture
    fn split(&mut self, width: u32, height: u32) {
        self.used = true;

        // Create right node if there's remaining width
        if self.width > width {
            self.right = Some(Box::new(PackingNode::new(
                self.x + width,
                self.y,
                self.width - width,
                height,
            )));
        }

        // Create down node if there's remaining height
        if self.height > height {
            self.down = Some(Box::new(PackingNode::new(
                self.x,
                self.y + height,
                self.width,
                self.height - height,
            )));
        }
    }
}

/// A single texture atlas that can hold multiple textures
pub struct TextureAtlas {
    pub id: u32,
    pub size: u32,
    pub texture: Option<Texture>,
    pub data: Vec<u8>,
    pub textures: HashMap<u32, AtlasTexture>,
    pub material_lookup: HashMap<MaterialType, u32>,
    packing_root: PackingNode,
    next_texture_id: u32,
    usage_count: usize,
}

impl TextureAtlas {
    pub fn new(id: u32, size: u32) -> Self {
        Self {
            id,
            size,
            texture: None,
            data: vec![0u8; (size * size * 4) as usize],
            textures: HashMap::new(),
            material_lookup: HashMap::new(),
            packing_root: PackingNode::new(0, 0, size, size),
            next_texture_id: 0,
            usage_count: 0,
        }
    }

    /// Add a texture to this atlas
    pub fn add_texture(&mut self, material_type: MaterialType, texture_size: u32, data: &[u8]) -> Option<u32> {
        // Find and allocate space in the atlas
        if let Some((x, y)) = self.packing_root.allocate(texture_size, texture_size) {
            let texture_id = self.next_texture_id;
            self.next_texture_id += 1;

            // Copy texture data into atlas
            self.copy_texture_data(x, y, texture_size, data);

            // Calculate UV coordinates
            let u_min = x as f32 / self.size as f32;
            let v_min = y as f32 / self.size as f32;
            let u_max = (x + texture_size) as f32 / self.size as f32;
            let v_max = (y + texture_size) as f32 / self.size as f32;

            let atlas_texture = AtlasTexture {
                id: texture_id,
                material_type,
                size: texture_size,
                atlas_id: self.id,
                uv_coords: AtlasUV::new(u_min, v_min, u_max, v_max),
                is_emissive: material_type.is_emissive(),
                is_transparent: !material_type.is_opaque(),
            };

            self.textures.insert(texture_id, atlas_texture);
            self.material_lookup.insert(material_type, texture_id);

            log::debug!("Added texture {} for material {:?} at ({}, {}) size {} to atlas {}",
                       texture_id, material_type, x, y, texture_size, self.id);

            Some(texture_id)
        } else {
            None
        }
    }

    /// Copy texture data into the atlas at the specified position
    fn copy_texture_data(&mut self, x: u32, y: u32, size: u32, data: &[u8]) {
        for row in 0..size {
            for col in 0..size {
                let src_index = ((row * size + col) * 4) as usize;
                let dst_x = x + col;
                let dst_y = y + row;
                let dst_index = ((dst_y * self.size + dst_x) * 4) as usize;

                if src_index + 3 < data.len() && dst_index + 3 < self.data.len() {
                    self.data[dst_index..dst_index + 4].copy_from_slice(&data[src_index..src_index + 4]);
                }
            }
        }
    }

    /// Create Metal texture from atlas data
    pub fn create_metal_texture(&mut self, device: &Device) -> Result<(), String> {
        let texture_descriptor = TextureDescriptor::new();
        texture_descriptor.set_texture_type(MTLTextureType::D2);
        texture_descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        texture_descriptor.set_width(self.size as u64);
        texture_descriptor.set_height(self.size as u64);
        texture_descriptor.set_usage(MTLTextureUsage::ShaderRead);

        let texture = device.new_texture(&texture_descriptor);

        let region = MTLRegion::new_2d(0, 0, self.size as u64, self.size as u64);
        texture.replace_region(region, 0, self.data.as_ptr() as *const std::ffi::c_void, (self.size * 4) as u64);

        self.texture = Some(texture);

        log::info!("Created Metal texture atlas {} ({}x{})", self.id, self.size, self.size);
        Ok(())
    }

    /// Get texture by material type
    pub fn get_texture_by_material(&self, material_type: MaterialType) -> Option<&AtlasTexture> {
        self.material_lookup.get(&material_type)
            .and_then(|&id| self.textures.get(&id))
    }

    /// Check if atlas has space for a texture of given size
    pub fn can_fit(&self, size: u32) -> bool {
        self.packing_root.can_fit(size, size)
    }

    /// Get utilization percentage
    pub fn utilization(&self) -> f32 {
        let used_pixels = self.textures.values()
            .map(|t| t.size * t.size)
            .sum::<u32>() as f32;
        let total_pixels = (self.size * self.size) as f32;
        (used_pixels / total_pixels) * 100.0
    }

    /// Increment usage count for performance tracking
    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
    }

    /// Get usage count
    pub fn get_usage_count(&self) -> usize {
        self.usage_count
    }
}

impl MaterialType {
    /// Check if material emits light
    pub fn is_emissive(&self) -> bool {
        matches!(self, MaterialType::Crystal | MaterialType::Lava)
    }
}

/// Dynamic texture atlas manager
pub struct DynamicTextureAtlas {
    atlases: Vec<TextureAtlas>,
    next_atlas_id: u32,
    material_to_atlas: HashMap<MaterialType, u32>,
    atlas_size: u32,
    stats: AtlasStats,
}

/// Performance statistics for texture atlas system
#[derive(Debug, Clone)]
pub struct AtlasStats {
    pub total_atlases: usize,
    pub total_textures: usize,
    pub memory_usage_mb: f32,
    pub average_utilization: f32,
    pub texture_switches_saved: usize,
}

impl DynamicTextureAtlas {
    pub fn new(atlas_size: u32) -> Self {
        Self {
            atlases: Vec::new(),
            next_atlas_id: 0,
            material_to_atlas: HashMap::new(),
            atlas_size: atlas_size.min(MAX_ATLAS_SIZE),
            stats: AtlasStats {
                total_atlases: 0,
                total_textures: 0,
                memory_usage_mb: 0.0,
                average_utilization: 0.0,
                texture_switches_saved: 0,
            },
        }
    }

    /// Add a material texture to the atlas system
    pub fn add_material_texture(&mut self, material_type: MaterialType, size: u32) -> Option<AtlasUV> {
        // Generate procedural texture data for the material
        let texture_data = self.generate_material_texture(material_type, size);

        // Try to find an existing atlas with space
        for atlas in &mut self.atlases {
            if atlas.can_fit(size) {
                if let Some(texture_id) = atlas.add_texture(material_type, size, &texture_data) {
                    let atlas_id = atlas.id;
                    let uv_coords = atlas.textures.get(&texture_id).map(|t| t.uv_coords);
                    self.material_to_atlas.insert(material_type, atlas_id);
                    self.update_stats();
                    return uv_coords;
                }
            }
        }

        // Create new atlas if no space found
        let atlas_id = self.next_atlas_id;
        self.next_atlas_id += 1;

        let mut new_atlas = TextureAtlas::new(atlas_id, self.atlas_size);
        if let Some(texture_id) = new_atlas.add_texture(material_type, size, &texture_data) {
            self.material_to_atlas.insert(material_type, atlas_id);
            let uv_coords = new_atlas.textures.get(&texture_id).unwrap().uv_coords;
            self.atlases.push(new_atlas);
            self.update_stats();

            log::info!("Created new texture atlas {} for material {:?}", atlas_id, material_type);
            Some(uv_coords)
        } else {
            log::error!("Failed to add texture to new atlas for material {:?}", material_type);
            None
        }
    }

    /// Generate procedural texture data for a material type
    fn generate_material_texture(&self, material_type: MaterialType, size: u32) -> Vec<u8> {
        let mut data = vec![0u8; (size * size * 4) as usize];
        let base_color = material_type.color();

        for y in 0..size {
            for x in 0..size {
                let index = ((y * size + x) * 4) as usize;

                // Add procedural noise for texture detail
                let noise = ((x * 7 + y * 11 + material_type as usize as u32 * 13) % 32) as f32 / 32.0;
                let brightness = 0.8 + noise * 0.4; // Range: 0.8 to 1.2

                data[index] = (base_color[0] * brightness).clamp(0.0, 1.0) as u8;
                data[index + 1] = (base_color[1] * brightness).clamp(0.0, 1.0) as u8;
                data[index + 2] = (base_color[2] * brightness).clamp(0.0, 1.0) as u8;
                data[index + 3] = if material_type.is_opaque() { 255 } else { 200 };
            }
        }

        data
    }

    /// Create all Metal textures for the atlases
    pub fn create_metal_textures(&mut self, device: &Device) -> Result<(), String> {
        for atlas in &mut self.atlases {
            atlas.create_metal_texture(device)?;
        }
        log::info!("Created {} Metal texture atlases", self.atlases.len());
        Ok(())
    }

    /// Get UV coordinates for a material
    pub fn get_material_uv(&mut self, material_type: MaterialType) -> Option<AtlasUV> {
        // Check if material already exists
        if let Some(&atlas_id) = self.material_to_atlas.get(&material_type) {
            // First, find and get the UV coordinates
            let uv_coords = self.atlases.iter()
                .find(|a| a.id == atlas_id)
                .and_then(|a| a.get_texture_by_material(material_type))
                .map(|t| t.uv_coords);

            // Then increment usage
            if let Some(atlas) = self.atlases.iter_mut().find(|a| a.id == atlas_id) {
                atlas.increment_usage();
            }

            if uv_coords.is_some() {
                return uv_coords;
            }
        }

        // Add material if not found
        self.add_material_texture(material_type, 64) // Default 64x64 texture size
    }

    /// Get Metal texture for a material type
    pub fn get_material_texture(&self, material_type: MaterialType) -> Option<&Texture> {
        if let Some(&atlas_id) = self.material_to_atlas.get(&material_type) {
            self.atlases.iter()
                .find(|a| a.id == atlas_id)
                .and_then(|a| a.texture.as_ref())
        } else {
            None
        }
    }

    /// Get all atlas textures (for binding to GPU)
    pub fn get_all_textures(&self) -> Vec<&Texture> {
        self.atlases.iter()
            .filter_map(|atlas| atlas.texture.as_ref())
            .collect()
    }

    /// Update performance statistics
    fn update_stats(&mut self) {
        self.stats.total_atlases = self.atlases.len();
        self.stats.total_textures = self.atlases.iter()
            .map(|a| a.textures.len())
            .sum();

        self.stats.memory_usage_mb = self.atlases.iter()
            .map(|a| (a.size * a.size * 4) as f32 / (1024.0 * 1024.0))
            .sum();

        if !self.atlases.is_empty() {
            self.stats.average_utilization = self.atlases.iter()
                .map(|a| a.utilization())
                .sum::<f32>() / self.atlases.len() as f32;
        }

        // Estimate texture switches saved (each atlas can batch multiple materials)
        self.stats.texture_switches_saved = self.atlases.iter()
            .map(|a| a.textures.len().saturating_sub(1))
            .sum();
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &AtlasStats {
        &self.stats
    }

    /// Pre-load all common material textures
    pub fn preload_materials(&mut self) -> Result<(), String> {
        let materials = [
            MaterialType::Stone, MaterialType::Earth, MaterialType::Water,
            MaterialType::Grass, MaterialType::Sand, MaterialType::Wood,
            MaterialType::Crystal, MaterialType::Lava, MaterialType::Air,
        ];

        for material in &materials {
            self.add_material_texture(*material, 64);
        }

        log::info!("Preloaded {} material textures", materials.len());
        Ok(())
    }

    /// Clear all atlases
    pub fn clear(&mut self) {
        self.atlases.clear();
        self.material_to_atlas.clear();
        self.next_atlas_id = 0;
        self.update_stats();
    }
}

impl Default for DynamicTextureAtlas {
    fn default() -> Self {
        Self::new(1024) // Default 1024x1024 atlases
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_uv_creation() {
        let uv = AtlasUV::new(0.0, 0.0, 0.5, 0.5);
        assert_eq!(uv.u_min, 0.0);
        assert_eq!(uv.v_min, 0.0);
        assert_eq!(uv.u_max, 0.5);
        assert_eq!(uv.v_max, 0.5);
    }

    #[test]
    fn test_atlas_texture_addition() {
        let mut atlas = TextureAtlas::new(0, 256);
        let texture_data = vec![255u8; 64 * 64 * 4]; // 64x64 white texture

        let texture_id = atlas.add_texture(MaterialType::Stone, 64, &texture_data);
        assert!(texture_id.is_some());
        assert_eq!(atlas.textures.len(), 1);
    }

    #[test]
    fn test_dynamic_atlas_material_loading() {
        let mut atlas_manager = DynamicTextureAtlas::new(512);

        let uv = atlas_manager.add_material_texture(MaterialType::Stone, 64);
        assert!(uv.is_some());

        let stats = atlas_manager.get_stats();
        assert_eq!(stats.total_textures, 1);
        assert_eq!(stats.total_atlases, 1);
    }

    #[test]
    fn test_material_texture_retrieval() {
        let mut atlas_manager = DynamicTextureAtlas::new(512);

        // Add a material
        atlas_manager.add_material_texture(MaterialType::Grass, 32);

        // Retrieve it
        let uv = atlas_manager.get_material_uv(MaterialType::Grass);
        assert!(uv.is_some());
    }

    #[test]
    fn test_atlas_utilization() {
        let mut atlas = TextureAtlas::new(0, 256);
        let texture_data = vec![255u8; 64 * 64 * 4];

        atlas.add_texture(MaterialType::Stone, 64, &texture_data);
        atlas.add_texture(MaterialType::Grass, 64, &texture_data);

        let utilization = atlas.utilization();
        assert!(utilization > 0.0);
        assert!(utilization <= 100.0);
    }
}