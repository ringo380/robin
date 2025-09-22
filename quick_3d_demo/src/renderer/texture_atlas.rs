// Texture atlas system for block materials
// Manages a 256x256 texture atlas with 16x16 tiles (256 total tiles)

use crate::game::VoxelType;

/// Size of the texture atlas (power of 2 for better GPU performance)
pub const ATLAS_SIZE: u32 = 256;

/// Size of each tile in the atlas
pub const TILE_SIZE: u32 = 16;

/// Number of tiles per row/column
pub const TILES_PER_ROW: u32 = ATLAS_SIZE / TILE_SIZE; // 16

/// Total number of tiles available
pub const TOTAL_TILES: u32 = TILES_PER_ROW * TILES_PER_ROW; // 256

/// UV coordinates for a tile in the atlas
#[derive(Debug, Clone, Copy)]
pub struct TileUV {
    /// UV coordinates for the four corners: [bottom-left, bottom-right, top-right, top-left]
    pub coords: [[f32; 2]; 4],
}

impl TileUV {
    /// Create UV coordinates for a tile at the given position
    pub fn new(tile_x: u32, tile_y: u32) -> Self {
        let u_start = (tile_x as f32) / (TILES_PER_ROW as f32);
        let v_start = (tile_y as f32) / (TILES_PER_ROW as f32);
        let u_end = ((tile_x + 1) as f32) / (TILES_PER_ROW as f32);
        let v_end = ((tile_y + 1) as f32) / (TILES_PER_ROW as f32);

        Self {
            coords: [
                [u_start, v_start], // bottom-left
                [u_end, v_start],   // bottom-right
                [u_end, v_end],     // top-right
                [u_start, v_end],   // top-left
            ],
        }
    }
}

/// Maps voxel types to their tile positions in the atlas
pub struct TextureAtlas {
    /// Mapping from voxel type to tile UV coordinates
    tile_mapping: [TileUV; 16], // Support for first 16 materials
}

impl TextureAtlas {
    /// Create a new texture atlas with predefined material mappings
    pub fn new() -> Self {
        let mut tile_mapping = [TileUV::new(0, 0); 16];

        // Map voxel types to specific tile positions
        // Using a 4x4 grid for the main materials in the top-left corner
        tile_mapping[VoxelType::Air as usize] = TileUV::new(0, 0);      // Air (transparent, not rendered)
        tile_mapping[VoxelType::Stone as usize] = TileUV::new(1, 0);    // Stone - gray
        tile_mapping[VoxelType::Dirt as usize] = TileUV::new(2, 0);     // Dirt - brown
        tile_mapping[VoxelType::Grass as usize] = TileUV::new(3, 0);    // Grass - green

        tile_mapping[VoxelType::Sand as usize] = TileUV::new(0, 1);     // Sand - tan
        tile_mapping[VoxelType::Water as usize] = TileUV::new(1, 1);    // Water - blue
        tile_mapping[VoxelType::Wood as usize] = TileUV::new(2, 1);     // Wood - brown
        tile_mapping[VoxelType::Leaves as usize] = TileUV::new(3, 1);   // Leaves - green

        tile_mapping[VoxelType::Crystal as usize] = TileUV::new(0, 2);  // Crystal - magenta (emissive)
        tile_mapping[VoxelType::Lava as usize] = TileUV::new(1, 2);     // Lava - red (emissive)

        // Future materials can use positions (0,2), (1,2), etc.

        Self { tile_mapping }
    }

    /// Get UV coordinates for a specific voxel type
    pub fn get_uv(&self, voxel_type: VoxelType) -> TileUV {
        let index = voxel_type as usize;
        if index < self.tile_mapping.len() {
            self.tile_mapping[index]
        } else {
            // Default to stone texture for unknown types
            self.tile_mapping[VoxelType::Stone as usize]
        }
    }

    /// Generate the texture atlas as raw RGBA8 data
    /// This creates a procedural texture atlas with distinct patterns for each material
    pub fn generate_atlas_data() -> Vec<u8> {
        let mut data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize];

        // Generate each tile
        for tile_y in 0..TILES_PER_ROW {
            for tile_x in 0..TILES_PER_ROW {
                let tile_index = tile_y * TILES_PER_ROW + tile_x;
                Self::generate_tile(&mut data, tile_x, tile_y, tile_index);
            }
        }

        data
    }

    /// Generate a single tile in the atlas
    fn generate_tile(data: &mut [u8], tile_x: u32, tile_y: u32, tile_index: u32) {
        let start_x = tile_x * TILE_SIZE;
        let start_y = tile_y * TILE_SIZE;

        // Define colors for different materials based on tile position
        // First row: Air(0,0), Stone(1,0), Dirt(2,0), Grass(3,0)
        // Second row: Sand(0,1), Water(1,1), Wood(2,1), Leaves(3,1)
        // Third row: Crystal(0,2), Lava(1,2), etc.
        let color = match (tile_x, tile_y) {
            (0, 0) => [0, 0, 0, 0],         // Air - Transparent (not actually rendered)
            (1, 0) => [128, 128, 128, 255], // Stone - Gray
            (2, 0) => [139, 69, 19, 255],   // Dirt - Brown
            (3, 0) => [34, 139, 34, 255],   // Grass - Green

            (0, 1) => [238, 203, 173, 255], // Sand - Tan
            (1, 1) => [0, 100, 200, 200],   // Water - Blue with transparency
            (2, 1) => [160, 82, 45, 255],   // Wood - Dark brown
            (3, 1) => [46, 125, 50, 255],   // Leaves - Dark green

            (0, 2) => [255, 0, 255, 255],   // Crystal - Magenta (emissive)
            (1, 2) => [255, 69, 0, 255],    // Lava - Red-orange (emissive)

            _ => [64, 64, 64, 255],         // Default - Dark gray
        };

        // Fill the tile with the base color and add some texture pattern
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let pixel_x = start_x + x;
                let pixel_y = start_y + y;
                let pixel_index = ((pixel_y * ATLAS_SIZE + pixel_x) * 4) as usize;

                if pixel_index + 3 < data.len() {
                    // Add some noise/pattern based on position
                    let noise = ((x * 7 + y * 11 + tile_index * 13) % 32) as u8;
                    let brightness_offset = (noise as i16 - 16) / 4; // Range: -4 to +4

                    data[pixel_index] = (color[0] as i16 + brightness_offset).clamp(0, 255) as u8; // R
                    data[pixel_index + 1] = (color[1] as i16 + brightness_offset).clamp(0, 255) as u8; // G
                    data[pixel_index + 2] = (color[2] as i16 + brightness_offset).clamp(0, 255) as u8; // B
                    data[pixel_index + 3] = color[3]; // A
                }
            }
        }

        // Add border for debugging (1 pixel border)
        if tile_x < 4 && tile_y < 3 { // Only for the main materials (4x3 grid)
            Self::add_tile_border(data, start_x, start_y);
        }
    }

    /// Add a subtle border around a tile for debugging
    fn add_tile_border(data: &mut [u8], start_x: u32, start_y: u32) {
        let border_color = [255, 255, 255, 64]; // Semi-transparent white

        for i in 0..TILE_SIZE {
            // Top and bottom borders
            let top_index = ((start_y * ATLAS_SIZE + start_x + i) * 4) as usize;
            let bottom_index = (((start_y + TILE_SIZE - 1) * ATLAS_SIZE + start_x + i) * 4) as usize;

            if top_index + 3 < data.len() {
                data[top_index..top_index + 4].copy_from_slice(&border_color);
            }
            if bottom_index + 3 < data.len() {
                data[bottom_index..bottom_index + 4].copy_from_slice(&border_color);
            }

            // Left and right borders
            let left_index = (((start_y + i) * ATLAS_SIZE + start_x) * 4) as usize;
            let right_index = (((start_y + i) * ATLAS_SIZE + start_x + TILE_SIZE - 1) * 4) as usize;

            if left_index + 3 < data.len() {
                data[left_index..left_index + 4].copy_from_slice(&border_color);
            }
            if right_index + 3 < data.len() {
                data[right_index..right_index + 4].copy_from_slice(&border_color);
            }
        }
    }
}

impl Default for TextureAtlas {
    fn default() -> Self {
        Self::new()
    }
}