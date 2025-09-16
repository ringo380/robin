/*!
 * Pixel Scatter System
 * 
 * Handles organic, pointillistic-style visual generation using point clouds,
 * scatter patterns, and organic distribution algorithms. Creates natural,
 * organic-looking visuals through strategic pixel/point placement.
 */

use crate::engine::{
    graphics::{Color, Texture, ParticleSystem},
    math::{Vec2, Vec3},
    error::{RobinError, RobinResult},
};
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSurface {
    pub surface_id: uuid::Uuid,
    pub vertices: Vec<Vec3>,
    pub texture_coords: Vec<Vec2>,
    pub normals: Vec<Vec3>,
    pub material_properties: MaterialProperties,
    pub scatter_pattern: ScatterPattern,
    #[serde(skip)]
    pub texture: Option<std::sync::Arc<Texture>>,
    #[serde(skip)]
    pub normal_map: Option<std::sync::Arc<Texture>>,
    pub point_cloud: PointCloud,
    pub cache_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaterialProperties {
    pub roughness: f32,
    pub metallic: f32,
    pub specular: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScatterPattern {
    Random,
    Organic,
    Fractal,
    Grid,
    Radial,
}

/// Core pixel scatter system
#[derive(Debug)]
pub struct PixelScatterSystem {
    /// Configuration
    config: PixelScatterConfig,
    /// Active scatter patterns
    patterns: HashMap<String, ScatterPattern>,
    /// Point cloud cache
    point_cache: HashMap<String, PointCloud>,
    /// Distribution algorithms
    distributors: ScatterDistributors,
    /// Organic shape generators
    shape_generators: OrganicShapeGenerators,
    /// Rendering system for scattered points
    renderer: ScatterRenderer,
}

impl PixelScatterSystem {
    pub fn new(config: PixelScatterConfig) -> Self {
        Self {
            patterns: HashMap::new(),
            point_cache: HashMap::new(),
            distributors: ScatterDistributors::new(),
            shape_generators: OrganicShapeGenerators::new(),
            renderer: ScatterRenderer::new(config.clone()),
            config,
        }
    }

    /// Generate a character using pixel scatter techniques
    pub fn generate_character(&mut self, params: CharacterParams) -> RobinResult<GeneratedCharacter> {
        let start_time = std::time::Instant::now();

        // Create base character shape using organic distribution
        let character_points = self.generate_character_points(&params)?;

        // Apply scatter patterns for different body parts
        let detailed_points = self.apply_character_details(character_points, &params)?;

        // Generate organic textures for skin, hair, clothing
        let textures = self.generate_character_textures(&params)?;

        // Create point cloud mesh
        let mesh = self.generate_point_cloud_mesh(&detailed_points)?;

        // Generate organic animations
        let animations = if params.generate_animations {
            self.generate_organic_animations(&detailed_points, &params)?
        } else {
            Vec::new()
        };

        let generation_time = start_time.elapsed().as_secs_f32();

        // Extract values before moving detailed_points
        let point_count = detailed_points.points.len();
        let organic_complexity = self.calculate_organic_complexity(&detailed_points);

        Ok(GeneratedCharacter {
            mesh,
            point_cloud: detailed_points,
            textures,
            animations,
            metadata: CharacterMetadata {
                character_type: self.parse_character_type(&params.character_type),
                generation_time,
                point_count,
                organic_complexity,
            },
            cache_key: params.get_cache_key(),
        })
    }

    /// Generate character details for hybrid mode
    pub fn generate_character_details(&mut self, params: CharacterParams) -> RobinResult<GeneratedCharacter> {
        // Generate only detail elements (hair, skin texture, clothing patterns)
        let detail_points = self.generate_character_detail_points(&params)?;
        let detail_textures = self.generate_detail_textures(&params)?;
        let mesh = self.generate_point_cloud_mesh(&detail_points)?;

        // Extract values before moving detail_points
        let point_count = detail_points.points.len();
        let organic_complexity = self.calculate_organic_complexity(&detail_points);

        Ok(GeneratedCharacter {
            mesh,
            point_cloud: detail_points,
            textures: detail_textures,
            animations: Vec::new(),
            metadata: CharacterMetadata {
                character_type: self.parse_character_type(&params.character_type),
                generation_time: 0.0,
                point_count,
                organic_complexity,
            },
            cache_key: format!("{}_details", params.get_cache_key()),
        })
    }

    /// Generate an environment using organic scatter techniques
    pub fn generate_environment(&mut self, params: EnvironmentParams, heightmap: Heightmap) -> RobinResult<GeneratedEnvironment> {
        let environment_points = match params.environment_type {
            EnvironmentType::Forest => {
                self.generate_forest_scatter(&params, &heightmap)?
            }
            EnvironmentType::Desert => {
                self.generate_desert_scatter(&params, &heightmap)?
            }
            EnvironmentType::Cave => {
                self.generate_cave_scatter(&params, &heightmap)?
            }
            EnvironmentType::Ocean => {
                self.generate_ocean_scatter(&params, &heightmap)?
            }
            EnvironmentType::Abstract => {
                self.generate_abstract_scatter(&params, &heightmap)?
            }
            _ => {
                self.generate_generic_scatter(&params, &heightmap)?
            }
        };

        // Generate atmospheric effects
        let atmosphere = self.generate_atmospheric_effects(&params)?;

        // Create environment textures
        let textures = self.generate_environment_textures(&params, &heightmap)?;

        // Generate mesh
        let mesh = self.generate_environment_mesh(&environment_points, &atmosphere)?;

        // Extract values before moving environment_points and atmosphere
        let point_count = environment_points.points.len();
        let atmosphere_density = atmosphere.density;
        let texture_count = textures.len();
        
        // Extract values from params before partial move
        let cache_key = params.get_cache_key();
        let environment_type = params.environment_type;

        Ok(GeneratedEnvironment {
            point_cloud: environment_points,
            atmosphere,
            textures,
            mesh,
            metadata: EnvironmentMetadata {
                environment_type,
                point_count,
                atmosphere_density,
                texture_count,
            },
            cache_key,
        })
    }

    /// Generate organic elements for hybrid environments
    pub fn generate_environment_organic(&mut self, params: EnvironmentParams, heightmap: Heightmap) -> RobinResult<GeneratedEnvironment> {
        // Generate only organic elements (vegetation, water, clouds, etc.)
        let organic_points = self.generate_organic_environment_elements(&params, &heightmap)?;
        let atmosphere = self.generate_atmospheric_effects(&params)?;
        let mesh = self.generate_environment_mesh(&organic_points, &atmosphere)?;

        // Extract values before moving organic_points and atmosphere
        let point_count = organic_points.points.len();
        let atmosphere_density = atmosphere.density;
        
        // Extract values from params before partial move
        let cache_key = format!("{}_organic", params.get_cache_key());
        let environment_type = params.environment_type;

        Ok(GeneratedEnvironment {
            point_cloud: organic_points,
            atmosphere,
            textures: Vec::new(),
            mesh,
            metadata: EnvironmentMetadata {
                environment_type,
                point_count,
                atmosphere_density,
                texture_count: 0,
            },
            cache_key,
        })
    }

    /// Generate a surface using scatter techniques
    pub fn generate_surface(&mut self, params: SurfaceParams) -> RobinResult<GeneratedSurface> {
        let surface_points = match params.surface_type {
            SurfaceType::Organic => {
                self.generate_organic_surface_scatter(&params)?
            }
            SurfaceType::Fabric => {
                self.generate_fabric_surface_scatter(&params)?
            }
            SurfaceType::Liquid => {
                self.generate_liquid_surface_scatter(&params)?
            }
            SurfaceType::Natural => {
                self.generate_natural_surface_scatter(&params)?
            }
            _ => {
                self.generate_generic_surface_scatter(&params)?
            }
        };

        // Generate surface texture from scatter pattern
        let texture = self.generate_texture_from_scatter(&surface_points, &params)?;
        
        // Generate normal map for lighting
        let normal_map = self.generate_scatter_normal_map(&surface_points)?;

        Ok(GeneratedSurface {
            surface_id: uuid::Uuid::new_v4(),
            vertices: vec![],
            texture_coords: vec![],
            normals: vec![],
            material_properties: self.calculate_scatter_material_properties(&params),
            scatter_pattern: ScatterPattern::Organic,
            texture: Some(std::sync::Arc::new(texture)),
            normal_map: Some(std::sync::Arc::new(normal_map)),
            point_cloud: surface_points,
            cache_key: params.get_cache_key(),
        })
    }

    /// Generate character points using organic distribution
    fn generate_character_points(&mut self, params: &CharacterParams) -> RobinResult<PointCloud> {
        let mut points = Vec::new();
        let base_density = params.detail_level.get_point_density();
        
        match params.character_type.to_lowercase().as_str() {
            "humanoid" => {
                self.generate_humanoid_scatter_points(&mut points, params, base_density)?;
            }
            "creature" => {
                self.generate_creature_scatter_points(&mut points, params, base_density)?;
            }
            "abstract" => {
                self.generate_abstract_scatter_points(&mut points, params, base_density)?;
            }
            "elemental" => {
                self.generate_elemental_scatter_points(&mut points, params, base_density)?;
            }
            _ => {
                self.generate_generic_scatter_points(&mut points, params, base_density)?;
            }
        }

        // Calculate bounds before moving points
        let bounds = self.calculate_point_bounds(&points);

        Ok(PointCloud {
            points,
            density: base_density,
            bounds,
            metadata: HashMap::new(),
        })
    }

    /// Generate distribution of scatter points
    pub fn generate_distribution(&mut self, params: &SurfaceParams) -> RobinResult<PointCloud> {
        let mut points = Vec::new();
        let base_density = 100.0; // Default density

        // Generate points based on surface type
        match params.surface_type {
            SurfaceType::Organic => {
                self.generate_organic_distribution(&mut points, params, base_density)?;
            }
            SurfaceType::Fabric => {
                self.generate_fabric_distribution(&mut points, params, base_density)?;
            }
            SurfaceType::Liquid => {
                self.generate_liquid_distribution(&mut points, params, base_density)?;
            }
            SurfaceType::Natural => {
                self.generate_natural_distribution(&mut points, params, base_density)?;
            }
            _ => {
                self.generate_generic_distribution(&mut points, params, base_density)?;
            }
        }

        // Calculate bounds before moving points
        let bounds = self.calculate_point_bounds(&points);

        Ok(PointCloud {
            points,
            density: base_density,
            bounds,
            metadata: HashMap::new(),
        })
    }

    // Distribution generation methods
    fn generate_organic_distribution(&mut self, points: &mut Vec<ScatterPoint>, _params: &SurfaceParams, density: f32) -> RobinResult<()> {
        // Placeholder: generate organic-style point distribution
        for i in 0..(density as usize) {
            points.push(ScatterPoint {
                position: Vec3::new(i as f32, 0.0, 0.0),
                color: Color::rgb(0.2, 0.8, 0.3),
                size: 1.0,
                intensity: 1.0,
                point_type: ScatterPointType::Organic,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 1.0,
            });
        }
        Ok(())
    }

    fn generate_fabric_distribution(&mut self, points: &mut Vec<ScatterPoint>, _params: &SurfaceParams, density: f32) -> RobinResult<()> {
        // Placeholder: generate fabric-style point distribution
        for i in 0..(density as usize) {
            points.push(ScatterPoint {
                position: Vec3::new(i as f32 * 0.5, 0.0, 0.0),
                color: Color::rgb(0.6, 0.4, 0.2),
                size: 0.8,
                intensity: 0.8,
                point_type: ScatterPointType::Fabric,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 1.0,
            });
        }
        Ok(())
    }

    fn generate_liquid_distribution(&mut self, points: &mut Vec<ScatterPoint>, _params: &SurfaceParams, density: f32) -> RobinResult<()> {
        // Placeholder: generate liquid-style point distribution
        for i in 0..(density as usize) {
            points.push(ScatterPoint {
                position: Vec3::new(i as f32 * 0.3, -1.0, 0.0),
                color: Color::rgb(0.2, 0.4, 0.8),
                size: 1.2,
                intensity: 0.9,
                point_type: ScatterPointType::Water,
                velocity: Vec3::new(0.0, -0.1, 0.0),
                lifetime: 2.0,
            });
        }
        Ok(())
    }

    fn generate_natural_distribution(&mut self, points: &mut Vec<ScatterPoint>, _params: &SurfaceParams, density: f32) -> RobinResult<()> {
        // Placeholder: generate natural-style point distribution
        for i in 0..(density as usize) {
            points.push(ScatterPoint {
                position: Vec3::new((i as f32).sin(), 0.0, (i as f32).cos()),
                color: Color::rgb(0.5, 0.7, 0.3),
                size: 0.9,
                intensity: 0.7,
                point_type: ScatterPointType::Foliage,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 1.5,
            });
        }
        Ok(())
    }

    fn generate_generic_distribution(&mut self, points: &mut Vec<ScatterPoint>, _params: &SurfaceParams, density: f32) -> RobinResult<()> {
        // Placeholder: generate generic point distribution
        for i in 0..(density as usize) {
            points.push(ScatterPoint {
                position: Vec3::new(i as f32, 0.0, 0.0),
                color: Color::rgb(0.5, 0.5, 0.5),
                size: 1.0,
                intensity: 0.5,
                point_type: ScatterPointType::Dust,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 1.0,
            });
        }
        Ok(())
    }

    /// Generate humanoid character using organic point distribution
    fn generate_humanoid_scatter_points(
        &mut self,
        points: &mut Vec<ScatterPoint>,
        params: &CharacterParams,
        density: f32
    ) -> RobinResult<()> {
        let height = params.scale * 180.0; // 180 units tall
        let primary_color = Color::rgb(0.8, 0.7, 0.6); // Default color, params.primary_color is String

        // Generate head using radial distribution
        let head_center = Vec3::new(0.0, height * 0.85, 0.0);
        let head_radius = height * 0.08;
        self.generate_organic_sphere_points(points, head_center, head_radius, primary_color, density * 1.2)?;

        // Generate torso using organic distribution
        let torso_center = Vec3::new(0.0, height * 0.5, 0.0);
        let torso_size = Vec3::new(height * 0.15, height * 0.25, height * 0.1);
        self.generate_organic_body_points(points, torso_center, torso_size, primary_color, density)?;

        // Generate arms with organic flow
        self.generate_arm_scatter_points(points, params, height, density)?;

        // Generate legs with muscle definition
        self.generate_leg_scatter_points(points, params, height, density)?;

        // Add hair if specified
        if params.has_hair {
            let hair_color = Color::rgb(0.3, 0.2, 0.1); // Default color, params.hair_color is String
            self.generate_hair_scatter_points(points, head_center, head_radius * 1.2, hair_color, density * 0.8)?;
        }

        // Add clothing scatter patterns
        for clothing_name in &params.clothing {
            // Convert string to ClothingItem for compatibility
            let clothing_item = ClothingItem {
                item_type: clothing_name.clone(),
                color: Color::rgb(0.5, 0.5, 0.5), // Default color
                material: "fabric".to_string(),
                style: "casual".to_string(),
            };
            self.generate_clothing_scatter_points(points, &clothing_item, height, density * 0.6)?;
        }

        Ok(())
    }

    /// Generate organic sphere points using Poisson disk sampling
    fn generate_organic_sphere_points(
        &mut self,
        points: &mut Vec<ScatterPoint>,
        center: Vec3,
        radius: f32,
        color: Color,
        density: f32
    ) -> RobinResult<()> {
        let point_count = (radius * radius * density) as usize;
        
        for _ in 0..point_count {
            // Generate random point on sphere with organic variation
            let theta = fastrand::f32() * std::f32::consts::PI * 2.0;
            let phi = fastrand::f32() * std::f32::consts::PI;
            
            // Add organic distortion using noise
            let noise_factor = self.sample_3d_noise(center) * 0.2;
            let effective_radius = radius * (1.0 + noise_factor);
            
            let x = center.x + effective_radius * phi.sin() * theta.cos();
            let y = center.y + effective_radius * phi.cos();
            let z = center.z + effective_radius * phi.sin() * theta.sin();

            let position = Vec3::new(x, y, z);
            
            // Calculate organic color variation
            let color_variation = self.calculate_organic_color_variation(position, color);
            
            points.push(ScatterPoint {
                position,
                color: color_variation,
                size: 1.0 + fastrand::f32() * 0.5, // Random size variation
                intensity: 0.8 + fastrand::f32() * 0.2,
                point_type: ScatterPointType::Organic,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 10.0,
            });
        }

        Ok(())
    }

    /// Generate organic body points with muscle definition
    fn generate_organic_body_points(
        &mut self,
        points: &mut Vec<ScatterPoint>,
        center: Vec3,
        size: Vec3,
        color: Color,
        density: f32
    ) -> RobinResult<()> {
        let point_count = (size.x * size.y * size.z * density) as usize;

        for _ in 0..point_count {
            // Generate random point within bounds
            let local_x = (fastrand::f32() - 0.5) * size.x;
            let local_y = (fastrand::f32() - 0.5) * size.y;
            let local_z = (fastrand::f32() - 0.5) * size.z;

            // Apply organic shaping using noise and distance fields
            let distance_from_center = Vec3::new(local_x, local_y, local_z).magnitude();
            let max_distance = size.magnitude() * 0.5;
            
            if distance_from_center < max_distance {
                // Add muscle definition using layered noise
                let muscle_definition = self.calculate_muscle_definition(
                    Vec3::new(local_x, local_y, local_z), 
                    size
                );

                // Only place point if it passes organic shape test
                if muscle_definition > 0.3 {
                    let world_pos = center + Vec3::new(local_x, local_y, local_z);
                    let varied_color = self.calculate_organic_color_variation(world_pos, color);

                    points.push(ScatterPoint {
                        position: world_pos,
                        color: varied_color,
                        size: 0.8 + muscle_definition * 0.4,
                        intensity: muscle_definition,
                        point_type: ScatterPointType::Organic,
                        velocity: Vec3::new(0.0, 0.0, 0.0),
                        lifetime: 5.0,
                    });
                }
            }
        }

        Ok(())
    }

    /// Generate hair using flowing scatter patterns
    fn generate_hair_scatter_points(
        &mut self,
        points: &mut Vec<ScatterPoint>,
        head_center: Vec3,
        hair_volume: f32,
        color: Color,
        density: f32
    ) -> RobinResult<()> {
        let hair_strands = (density * 50.0) as usize;

        for _ in 0..hair_strands {
            // Generate hair strand starting from scalp
            let scalp_angle = fastrand::f32() * std::f32::consts::PI * 2.0;
            let scalp_elevation = fastrand::f32() * std::f32::consts::PI * 0.3;
            
            let start_x = head_center.x + (hair_volume * 0.7) * scalp_elevation.sin() * scalp_angle.cos();
            let start_y = head_center.y + (hair_volume * 0.7) * scalp_elevation.cos();
            let start_z = head_center.z + (hair_volume * 0.7) * scalp_elevation.sin() * scalp_angle.sin();

            let strand_start = Vec3::new(start_x, start_y, start_z);

            // Generate flowing hair strand
            self.generate_hair_strand_points(
                points,
                strand_start,
                hair_volume,
                color,
                15 // Points per strand
            )?;
        }

        Ok(())
    }

    /// Generate flowing points along a hair strand
    fn generate_hair_strand_points(
        &mut self,
        points: &mut Vec<ScatterPoint>,
        start: Vec3,
        max_length: f32,
        color: Color,
        strand_points: usize
    ) -> RobinResult<()> {
        let mut current_pos = start;
        let flow_direction = Vec3::new(0.0, -1.0, 0.0); // Hair flows downward
        let segment_length = max_length / strand_points as f32;

        for i in 0..strand_points {
            // Add organic flow and curl to hair
            let flow_noise = self.sample_3d_noise(current_pos * 0.1) * 0.3;
            let curl_factor = (i as f32 / strand_points as f32) * 0.5; // More curl at the ends
            
            let flow_variation = Vec3::new(
                flow_noise * curl_factor,
                0.0,
                (current_pos.x * 0.1).sin() * curl_factor * 0.2
            );

            current_pos = current_pos + (flow_direction + flow_variation) * segment_length;

            // Vary hair color along strand
            let strand_progress = i as f32 / strand_points as f32;
            let strand_color = self.interpolate_hair_color(color, strand_progress);

            points.push(ScatterPoint {
                position: current_pos,
                color: strand_color,
                size: 0.5 + (1.0 - strand_progress) * 0.3, // Thicker at root
                intensity: 1.0 - strand_progress * 0.3,
                point_type: ScatterPointType::Hair,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 8.0,
            });
        }

        Ok(())
    }

    /// Generate forest environment scatter
    fn generate_forest_scatter(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<PointCloud> {
        let mut points = Vec::new();
        let area = 1024.0 * 1024.0; // TODO: Get actual world size from heightmap or other source
        let base_density = 0.1; // Default density value

        // Generate trees using clustered distribution
        let tree_count = (area * base_density * 0.01) as usize;
        for _ in 0..tree_count {
            let x = fastrand::f32() * 1024.0; // TODO: Use actual world dimensions
            let z = fastrand::f32() * 1024.0; // TODO: Use actual world dimensions
            let y = heightmap.get_height(x as usize, z as usize);

            self.generate_tree_scatter_points(&mut points, Vec3::new(x, y, z), params)?;
        }

        // Generate undergrowth using organic distribution
        let undergrowth_density = base_density * 10.0;
        self.generate_undergrowth_scatter(&mut points, params, heightmap, undergrowth_density)?;

        // Generate atmospheric particles (pollen, dust, etc.)
        self.generate_forest_atmosphere(&mut points, params, area)?;

        Ok(PointCloud {
            points,
            density: base_density,
            bounds: (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1024.0, 256.0, 1024.0) // TODO: Use actual world dimensions (width, height, depth)
            ),
            metadata: HashMap::new(),
        })
    }

    /// Generate tree using organic scatter patterns
    fn generate_tree_scatter_points(
        &mut self,
        points: &mut Vec<ScatterPoint>,
        base_position: Vec3,
        params: &EnvironmentParams
    ) -> RobinResult<()> {
        let tree_height = 20.0 + fastrand::f32() * 30.0;
        let trunk_radius = 0.5 + fastrand::f32() * 1.5;
        let canopy_radius = 5.0 + fastrand::f32() * 10.0;

        // Generate trunk using organic distribution
        let trunk_points = (tree_height * trunk_radius * 20.0) as usize;
        for i in 0..trunk_points {
            let height_progress = i as f32 / trunk_points as f32;
            let y = base_position.y + height_progress * tree_height;
            
            // Add organic bark texture
            let bark_noise = self.sample_3d_noise(Vec3::new(base_position.x, y * 0.1, base_position.z));
            let radius_variation = trunk_radius * (1.0 + bark_noise * 0.3);
            
            let angle = fastrand::f32() * std::f32::consts::PI * 2.0;
            let radius = fastrand::f32() * radius_variation;
            
            let x = base_position.x + radius * angle.cos();
            let z = base_position.z + radius * angle.sin();

            points.push(ScatterPoint {
                position: Vec3::new(x, y, z),
                color: Color::rgb(0.4, 0.2, 0.1), // Brown bark
                size: 1.0 + bark_noise * 0.5,
                intensity: 0.8,
                point_type: ScatterPointType::Bark,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 15.0,
            });
        }

        // Generate canopy using clustered leaf distribution
        let canopy_center = Vec3::new(base_position.x, base_position.y + tree_height * 0.8, base_position.z);
        let leaf_points = (canopy_radius * canopy_radius * 100.0) as usize;

        for _ in 0..leaf_points {
            // Generate clustered leaf positions
            let cluster_offset = Vec3::new(
                (fastrand::f32() - 0.5) * canopy_radius * 2.0,
                (fastrand::f32() - 0.5) * canopy_radius * 0.8,
                (fastrand::f32() - 0.5) * canopy_radius * 2.0,
            );

            let leaf_pos = canopy_center + cluster_offset;
            
            // Check if point is within organic canopy shape
            let distance_from_center = cluster_offset.magnitude();
            let canopy_noise = self.sample_3d_noise(leaf_pos * 0.05);
            let effective_radius = canopy_radius * (1.0 + canopy_noise * 0.4);

            if distance_from_center < effective_radius {
                // Seasonal color variation
                let leaf_color = self.calculate_seasonal_leaf_color(leaf_pos);

                points.push(ScatterPoint {
                    position: leaf_pos,
                    color: leaf_color,
                    size: 0.5 + fastrand::f32() * 0.3,
                    intensity: 0.9 + canopy_noise * 0.1,
                    point_type: ScatterPointType::Foliage,
                    velocity: Vec3::new(0.0, 0.0, 0.0),
                    lifetime: 6.0,
                });
            }
        }

        Ok(())
    }

    // Helper methods for calculations and utilities
    fn sample_3d_noise(&self, position: Vec3) -> f32 {
        // Simplified 3D noise function - would use proper Perlin/Simplex noise
        (position.x.sin() * position.y.cos() * position.z.sin()).abs()
    }

    fn calculate_organic_color_variation(&self, position: Vec3, base_color: Color) -> Color {
        let noise = self.sample_3d_noise(position * 0.1);
        let variation = 0.1;
        
        Color::rgba(
            (base_color.r + (noise - 0.5) * variation).clamp(0.0, 1.0),
            (base_color.g + (noise - 0.5) * variation).clamp(0.0, 1.0),
            (base_color.b + (noise - 0.5) * variation).clamp(0.0, 1.0),
            base_color.a
        )
    }

    fn calculate_muscle_definition(&self, local_pos: Vec3, body_size: Vec3) -> f32 {
        // Simplified muscle definition calculation
        let distance_from_surface = local_pos.magnitude() / body_size.magnitude();
        let muscle_noise = self.sample_3d_noise(local_pos);
        (1.0 - distance_from_surface) * (0.5 + muscle_noise * 0.5)
    }

    fn interpolate_hair_color(&self, base_color: Color, progress: f32) -> Color {
        // Simple color interpolation for hair gradient
        let fade_factor = 1.0 - progress * 0.2;
        Color::rgba(
            base_color.r * fade_factor,
            base_color.g * fade_factor,
            base_color.b * fade_factor,
            base_color.a
        )
    }

    fn calculate_seasonal_leaf_color(&self, position: Vec3) -> Color {
        // Generate seasonal leaf colors
        let season_noise = self.sample_3d_noise(position * 0.02);
        
        if season_noise < 0.3 {
            Color::rgb(0.2, 0.8, 0.2) // Spring green
        } else if season_noise < 0.6 {
            Color::rgb(0.1, 0.6, 0.1) // Summer green
        } else if season_noise < 0.8 {
            Color::rgb(0.8, 0.6, 0.1) // Autumn yellow
        } else {
            Color::rgb(0.7, 0.3, 0.1) // Autumn red
        }
    }

    fn calculate_point_bounds(&self, points: &[ScatterPoint]) -> (Vec3, Vec3) {
        if points.is_empty() {
            return (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
        }

        let mut min = points[0].position;
        let mut max = points[0].position;

        for point in points.iter() {
            min.x = min.x.min(point.position.x);
            min.y = min.y.min(point.position.y);
            min.z = min.z.min(point.position.z);
            max.x = max.x.max(point.position.x);
            max.y = max.y.max(point.position.y);
            max.z = max.z.max(point.position.z);
        }

        (min, max)
    }

    fn calculate_organic_complexity(&self, point_cloud: &PointCloud) -> f32 {
        // Calculate complexity based on point distribution patterns
        let mut complexity = 0.0;
        
        // Factor in point count
        complexity += (point_cloud.points.len() as f32).sqrt() * 0.01;
        
        // Factor in point type diversity
        let mut point_types = std::collections::HashSet::new();
        for point in &point_cloud.points {
            point_types.insert(std::mem::discriminant(&point.point_type));
        }
        complexity += point_types.len() as f32 * 0.1;

        complexity.clamp(0.0, 1.0)
    }

    pub fn is_active(&self) -> bool {
        !self.patterns.is_empty() || !self.point_cache.is_empty()
    }

    pub fn get_memory_usage(&self) -> usize {
        self.point_cache.values().map(|pc| pc.points.len() * 64).sum::<usize>() +
        self.patterns.len() * 1024
    }

    // Placeholder implementations for methods that would be fully implemented
    fn apply_character_details(&mut self, points: PointCloud, params: &CharacterParams) -> RobinResult<PointCloud> { Ok(points) }
    fn generate_character_textures(&mut self, params: &CharacterParams) -> RobinResult<Vec<Texture>> { Ok(Vec::new()) }
    fn generate_point_cloud_mesh(&mut self, points: &PointCloud) -> RobinResult<ScatterMesh> { Ok(ScatterMesh::default()) }
    fn generate_organic_animations(&mut self, points: &PointCloud, params: &CharacterParams) -> RobinResult<Vec<OrganicAnimation>> { Ok(Vec::new()) }
    fn generate_character_detail_points(&mut self, params: &CharacterParams) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_detail_textures(&mut self, params: &CharacterParams) -> RobinResult<Vec<Texture>> { Ok(Vec::new()) }
    fn generate_desert_scatter(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_cave_scatter(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_ocean_scatter(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_abstract_scatter(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_generic_scatter(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_atmospheric_effects(&mut self, params: &EnvironmentParams) -> RobinResult<AtmosphereData> { Ok(AtmosphereData::default()) }
    fn generate_environment_textures(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<Vec<Texture>> { Ok(Vec::new()) }
    fn generate_environment_mesh(&mut self, points: &PointCloud, atmosphere: &AtmosphereData) -> RobinResult<ScatterMesh> { Ok(ScatterMesh::default()) }
    fn generate_organic_environment_elements(&mut self, params: &EnvironmentParams, heightmap: &Heightmap) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_organic_surface_scatter(&mut self, params: &SurfaceParams) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_fabric_surface_scatter(&mut self, params: &SurfaceParams) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_liquid_surface_scatter(&mut self, params: &SurfaceParams) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_natural_surface_scatter(&mut self, params: &SurfaceParams) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_generic_surface_scatter(&mut self, params: &SurfaceParams) -> RobinResult<PointCloud> { Ok(PointCloud::default()) }
    fn generate_texture_from_scatter(&mut self, points: &PointCloud, params: &SurfaceParams) -> RobinResult<Texture> { Ok(Texture::default()) }
    fn generate_scatter_normal_map(&mut self, points: &PointCloud) -> RobinResult<Texture> { Ok(Texture::default()) }
    fn calculate_scatter_material_properties(&mut self, params: &SurfaceParams) -> MaterialProperties { MaterialProperties::default() }
    fn generate_creature_scatter_points(&mut self, points: &mut Vec<ScatterPoint>, params: &CharacterParams, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_abstract_scatter_points(&mut self, points: &mut Vec<ScatterPoint>, params: &CharacterParams, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_elemental_scatter_points(&mut self, points: &mut Vec<ScatterPoint>, params: &CharacterParams, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_generic_scatter_points(&mut self, points: &mut Vec<ScatterPoint>, params: &CharacterParams, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_arm_scatter_points(&mut self, points: &mut Vec<ScatterPoint>, params: &CharacterParams, height: f32, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_leg_scatter_points(&mut self, points: &mut Vec<ScatterPoint>, params: &CharacterParams, height: f32, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_clothing_scatter_points(&mut self, points: &mut Vec<ScatterPoint>, clothing: &ClothingItem, height: f32, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_undergrowth_scatter(&mut self, points: &mut Vec<ScatterPoint>, params: &EnvironmentParams, heightmap: &Heightmap, density: f32) -> RobinResult<()> { Ok(()) }
    fn generate_forest_atmosphere(&mut self, points: &mut Vec<ScatterPoint>, params: &EnvironmentParams, area: f32) -> RobinResult<()> { Ok(()) }
}

/// Configuration for pixel scatter system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelScatterConfig {
    /// Maximum points per cloud
    pub max_points_per_cloud: usize,
    /// Point size range
    pub point_size_range: (f32, f32),
    /// Enable organic distribution algorithms
    pub organic_distribution: bool,
    /// Quality vs performance balance
    pub quality_factor: f32,
}

impl Default for PixelScatterConfig {
    fn default() -> Self {
        Self {
            max_points_per_cloud: 100000,
            point_size_range: (0.5, 2.0),
            organic_distribution: true,
            quality_factor: 0.8,
        }
    }
}

/// A cloud of scattered points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloud {
    pub points: Vec<ScatterPoint>,
    pub density: f32,
    pub bounds: (Vec3, Vec3),
    pub metadata: HashMap<String, String>,
}

impl Default for PointCloud {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            density: 1.0,
            bounds: (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
            metadata: HashMap::new(),
        }
    }
}

/// Individual scattered point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterPoint {
    pub position: Vec3,
    pub color: Color,
    pub size: f32,
    pub intensity: f32,
    pub point_type: ScatterPointType,
    pub velocity: Vec3,
    pub lifetime: f32,
}

/// Types of scatter points for different rendering behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScatterPointType {
    Organic,
    Hair,
    Bark,
    Foliage,
    Water,
    Dust,
    Fabric,
    Metal,
}

/// Scatter pattern for reusable distributions
#[derive(Debug, Clone)]
pub struct ScatterDistribution {
    pub name: String,
    pub points: Vec<Vec3>,
    pub distribution_type: DistributionType,
}

/// Different distribution algorithms
#[derive(Debug, Clone, Copy)]
pub enum DistributionType {
    Random,
    PoissonDisk,
    Organic,
    Clustered,
    Radial,
    Flow,
}

/// Atmospheric effects data
#[derive(Debug, Clone)]
pub struct AtmosphereData {
    pub density: f32,
    pub particles: Vec<AtmosphereParticle>,
    pub wind_direction: Vec3,
    pub turbulence: f32,
}

impl Default for AtmosphereData {
    fn default() -> Self {
        Self {
            density: 1.0,
            particles: Vec::new(),
            wind_direction: Vec3::new(1.0, 0.0, 0.0),
            turbulence: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtmosphereParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: Color,
    pub lifetime: f32,
}

/// Generated environment from scatter system
#[derive(Debug)]
pub struct GeneratedEnvironment {
    pub point_cloud: PointCloud,
    pub atmosphere: AtmosphereData,
    pub textures: Vec<Texture>,
    pub mesh: ScatterMesh,
    pub metadata: EnvironmentMetadata,
    pub cache_key: String,
}

#[derive(Debug, Clone)]
pub struct EnvironmentMetadata {
    pub environment_type: EnvironmentType,
    pub point_count: usize,
    pub atmosphere_density: f32,
    pub texture_count: usize,
}

/// Mesh generated from scattered points
#[derive(Debug, Clone, Default)]
pub struct ScatterMesh {
    pub point_vertices: Vec<PointVertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct PointVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub size: f32,
    pub intensity: f32,
}

/// Generated character from scatter system (specialized version)
#[derive(Debug)]
pub struct GeneratedCharacter {
    pub mesh: ScatterMesh,
    pub point_cloud: PointCloud,
    pub textures: Vec<Texture>,
    pub animations: Vec<OrganicAnimation>,
    pub metadata: CharacterMetadata,
    pub cache_key: String,
}

#[derive(Debug, Clone)]
pub struct CharacterMetadata {
    pub character_type: CharacterType,
    pub generation_time: f32,
    pub point_count: usize,
    pub organic_complexity: f32,
}

#[derive(Debug, Clone)]
pub struct OrganicAnimation {
    pub name: String,
    pub frames: Vec<OrganicFrame>,
    pub duration: f32,
}

#[derive(Debug, Clone)]
pub struct OrganicFrame {
    pub timestamp: f32,
    pub point_transforms: Vec<PointTransform>,
}

#[derive(Debug, Clone)]
pub struct PointTransform {
    pub point_index: usize,
    pub new_position: Vec3,
    pub new_color: Option<Color>,
    pub new_size: Option<f32>,
}

// Placeholder implementations for required systems
#[derive(Debug)]
pub struct ScatterDistributors;
impl ScatterDistributors { pub fn new() -> Self { Self } }

#[derive(Debug)]
pub struct OrganicShapeGenerators;
impl OrganicShapeGenerators { pub fn new() -> Self { Self } }

#[derive(Debug)]
pub struct ScatterRenderer { _config: PixelScatterConfig }
impl ScatterRenderer { 
    pub fn new(config: PixelScatterConfig) -> Self { Self { _config: config } }
}

// Import required types from other modules
pub use crate::engine::generation::{
    CharacterParams, EnvironmentParams, SurfaceParams,
    CharacterType, EnvironmentType, SurfaceType,
    DetailLevel, ClothingItem,
    Heightmap,
};

impl PixelScatterSystem {
    /// Parse character type string to enum
    fn parse_character_type(&self, character_type: &str) -> CharacterType {
        use crate::engine::generation::voxel_system::CharacterType;
        match character_type.to_lowercase().as_str() {
            "humanoid" => CharacterType::Humanoid,
            "creature" => CharacterType::Creature,
            "robot" => CharacterType::Robot,
            "abstract" => CharacterType::Abstract,
            "elemental" => CharacterType::Elemental,
            _ => CharacterType::Humanoid, // default
        }
    }
}

impl DetailLevel {
    pub fn get_point_density(&self) -> f32 {
        match self {
            DetailLevel::Low => 0.5,
            DetailLevel::Medium => 1.0,
            DetailLevel::High => 2.0,
            DetailLevel::Ultra => 4.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_cloud_creation() {
        let cloud = PointCloud::default();
        assert_eq!(cloud.points.len(), 0);
        assert_eq!(cloud.density, 0.0);
    }

    #[test]
    fn test_scatter_system_creation() {
        let config = PixelScatterConfig::default();
        let system = PixelScatterSystem::new(config);
        assert!(!system.is_active());
    }

    #[test]
    fn test_organic_color_variation() {
        let config = PixelScatterConfig::default();
        let system = PixelScatterSystem::new(config);
        let base_color = Color::rgb(1.0, 0.0, 0.0);
        let position = Vec3::new(10.0, 20.0, 30.0);
        
        let varied_color = system.calculate_organic_color_variation(position, base_color);
        
        // Color should be similar but slightly varied
        assert!(varied_color.r >= 0.8 && varied_color.r <= 1.0);
        assert!(varied_color.g >= 0.0 && varied_color.g <= 0.2);
        assert!(varied_color.b >= 0.0 && varied_color.b <= 0.2);
    }

    #[test]
    fn test_point_bounds_calculation() {
        let config = PixelScatterConfig::default();
        let system = PixelScatterSystem::new(config);
        
        let points = vec![
            ScatterPoint {
                position: Vec3::new(0.0, 0.0, 0.0),
                color: Color::rgb(1.0, 1.0, 1.0),
                size: 1.0,
                intensity: 1.0,
                point_type: ScatterPointType::Organic,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 1.0,
            },
            ScatterPoint {
                position: Vec3::new(10.0, 5.0, 3.0),
                color: Color::rgb(1.0, 1.0, 1.0),
                size: 1.0,
                intensity: 1.0,
                point_type: ScatterPointType::Organic,
                velocity: Vec3::new(0.0, 0.0, 0.0),
                lifetime: 1.0,
            },
        ];
        
        let (min, max) = system.calculate_point_bounds(&points);
        assert_eq!(min, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(max, Vec3::new(10.0, 5.0, 3.0));
    }
}