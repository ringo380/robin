/*!
 * Robin Engine Noise Generation System
 * 
 * Advanced noise generation utilities for procedural content creation,
 * including terrain heightmaps, texture patterns, and organic distributions.
 */

use crate::engine::{
    graphics::{Texture, Color},
    math::{Vec2, Vec3},
    error::{RobinError, RobinResult},
};
// Types defined in this file - no need to import
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main noise generation system
#[derive(Debug)]
pub struct NoiseSystem {
    config: NoiseConfig,
    generators: HashMap<String, NoiseGenerator>,
    patterns: HashMap<String, NoisePattern>,
    heightmap_cache: HashMap<String, Heightmap>,
    texture_cache: HashMap<String, Vec<u8>>,
}

impl NoiseSystem {
    pub fn new(config: NoiseConfig) -> Self {
        let mut system = Self {
            config,
            generators: HashMap::new(),
            patterns: HashMap::new(),
            heightmap_cache: HashMap::new(),
            texture_cache: HashMap::new(),
        };

        system.initialize_generators();
        system.load_default_patterns();
        system
    }

    /// Generate heightmap for terrain
    pub fn generate_heightmap(&mut self, params: TerrainParams) -> RobinResult<Heightmap> {
        let cache_key = format!("{}_{}_{}_{}", 
            params.width, params.height, params.scale, params.seed);

        if let Some(cached) = self.heightmap_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let heightmap = match params.noise_type {
            NoiseType::Perlin => self.generate_perlin_heightmap(params)?,
            NoiseType::Simplex => self.generate_simplex_heightmap(params)?,
            NoiseType::Ridged => self.generate_ridged_heightmap(params)?,
            NoiseType::Fractal => self.generate_fractal_heightmap(params)?,
            NoiseType::Voronoi => self.generate_voronoi_heightmap(params)?,
            NoiseType::Turbulence => self.generate_turbulence_heightmap(params)?,
        };

        self.heightmap_cache.insert(cache_key, heightmap.clone());
        Ok(heightmap)
    }

    /// Generate procedural texture
    pub fn generate_procedural_texture(&mut self, params: SurfaceParams) -> RobinResult<GeneratedSurface> {
        let texture_data = match &params.pattern {
            TexturePattern::Marble => self.generate_marble_texture(params.clone())?,
            TexturePattern::Wood => self.generate_wood_texture(params.clone())?,
            TexturePattern::Stone => self.generate_stone_texture(params.clone())?,
            TexturePattern::Fabric => self.generate_fabric_texture(params.clone())?,
            TexturePattern::Metal => self.generate_metal_texture(params.clone())?,
            TexturePattern::Organic => self.generate_organic_texture(params.clone())?,
            TexturePattern::Abstract => self.generate_abstract_texture(params.clone())?,
            TexturePattern::Glass => self.generate_glass_texture(params.clone())?,
        };

        Ok(GeneratedSurface {
            texture_data,
            normal_map: self.generate_normal_map(&params)?,
            roughness_map: self.generate_roughness_map(&params)?,
            properties: SurfaceProperties::from_params(&params),
        })
    }

    /// Generate noise-based distribution for scatter systems
    pub fn generate_scatter_distribution(&self, params: ScatterDistributionParams) -> RobinResult<Vec<Vec3>> {
        let mut points = Vec::new();
        let generator = self.get_generator(&params.noise_type)?;

        for _ in 0..params.point_count {
            let x = rand::random::<f32>() * params.bounds.x;
            let y = rand::random::<f32>() * params.bounds.y;
            let z = rand::random::<f32>() * params.bounds.z;

            // Use noise to determine if point should be placed
            let noise_value = generator.sample_3d(x, y, z, params.scale, params.seed);
            if noise_value > params.threshold {
                // Apply distribution bias
                let biased_position = self.apply_distribution_bias(
                    Vec3::new(x, y, z),
                    &params.distribution_type,
                    noise_value
                );
                points.push(biased_position);
            }
        }

        Ok(points)
    }

    /// Generate noise-based color patterns
    pub fn generate_noise_colors(&self, params: ColorNoiseParams) -> RobinResult<Vec<Color>> {
        let mut colors = Vec::new();
        let generator = self.get_generator(&params.noise_type)?;

        for i in 0..params.sample_count {
            let x = i as f32 / params.sample_count as f32 * params.frequency;
            let noise_value = generator.sample_1d(x, params.seed);
            
            let color = self.noise_to_color(noise_value, &params.color_palette, params.contrast);
            colors.push(color);
        }

        Ok(colors)
    }

    /// Generate volumetric noise for 3D effects
    pub fn generate_volumetric_noise(&self, params: VolumetricParams) -> RobinResult<Vec<f32>> {
        let mut volume = Vec::new();
        let generator = self.get_generator(&params.noise_type)?;

        let (width, height, depth) = (params.dimensions.x as usize, params.dimensions.y as usize, params.dimensions.z as usize);
        volume.reserve(width * height * depth);

        for z in 0..depth {
            for y in 0..height {
                for x in 0..width {
                    let noise_value = generator.sample_3d(
                        x as f32 * params.scale,
                        y as f32 * params.scale,
                        z as f32 * params.scale,
                        params.scale,
                        params.seed,
                    );
                    
                    let processed_value = self.process_volumetric_value(noise_value, &params.processing);
                    volume.push(processed_value);
                }
            }
        }

        Ok(volume)
    }

    fn initialize_generators(&mut self) {
        self.generators.insert("perlin".to_string(), NoiseGenerator::new(NoiseType::Perlin));
        self.generators.insert("simplex".to_string(), NoiseGenerator::new(NoiseType::Simplex));
        self.generators.insert("ridged".to_string(), NoiseGenerator::new(NoiseType::Ridged));
        self.generators.insert("fractal".to_string(), NoiseGenerator::new(NoiseType::Fractal));
        self.generators.insert("voronoi".to_string(), NoiseGenerator::new(NoiseType::Voronoi));
        self.generators.insert("turbulence".to_string(), NoiseGenerator::new(NoiseType::Turbulence));
    }

    fn load_default_patterns(&mut self) {
        // Terrain patterns
        self.patterns.insert("mountains".to_string(), NoisePattern {
            name: "Mountain Terrain".to_string(),
            layers: vec![
                NoiseLayer {
                    noise_type: NoiseType::Ridged,
                    frequency: 0.005,
                    amplitude: 1.0,
                    octaves: 6,
                },
                NoiseLayer {
                    noise_type: NoiseType::Perlin,
                    frequency: 0.02,
                    amplitude: 0.3,
                    octaves: 4,
                },
            ],
            post_processing: vec![
                PostProcessing::Sharpen(0.2),
                PostProcessing::Clamp(0.0, 1.0),
            ],
        });

        self.patterns.insert("rolling_hills".to_string(), NoisePattern {
            name: "Rolling Hills".to_string(),
            layers: vec![
                NoiseLayer {
                    noise_type: NoiseType::Perlin,
                    frequency: 0.01,
                    amplitude: 0.8,
                    octaves: 4,
                },
                NoiseLayer {
                    noise_type: NoiseType::Simplex,
                    frequency: 0.05,
                    amplitude: 0.2,
                    octaves: 2,
                },
            ],
            post_processing: vec![
                PostProcessing::Smooth(0.1),
                PostProcessing::Bias(0.1),
            ],
        });

        // Texture patterns
        self.patterns.insert("wood_grain".to_string(), NoisePattern {
            name: "Wood Grain".to_string(),
            layers: vec![
                NoiseLayer {
                    noise_type: NoiseType::Perlin,
                    frequency: 0.1,
                    amplitude: 1.0,
                    octaves: 3,
                },
                NoiseLayer {
                    noise_type: NoiseType::Turbulence,
                    frequency: 0.3,
                    amplitude: 0.2,
                    octaves: 2,
                },
            ],
            post_processing: vec![
                PostProcessing::WoodGrain,
                PostProcessing::Contrast(1.2),
            ],
        });

        self.patterns.insert("marble_veins".to_string(), NoisePattern {
            name: "Marble Veins".to_string(),
            layers: vec![
                NoiseLayer {
                    noise_type: NoiseType::Perlin,
                    frequency: 0.02,
                    amplitude: 1.0,
                    octaves: 4,
                },
                NoiseLayer {
                    noise_type: NoiseType::Turbulence,
                    frequency: 0.1,
                    amplitude: 0.5,
                    octaves: 3,
                },
            ],
            post_processing: vec![
                PostProcessing::MarbleVeins,
                PostProcessing::Smooth(0.05),
            ],
        });
    }

    fn generate_perlin_heightmap(&self, params: TerrainParams) -> RobinResult<Heightmap> {
        let generator = self.get_generator(&NoiseType::Perlin)?;
        let mut heights = vec![vec![0.0; params.height]; params.width];

        for x in 0..params.width {
            for y in 0..params.height {
                let noise_value = generator.sample_2d(
                    x as f32 * params.scale,
                    y as f32 * params.scale,
                    params.scale,
                    params.seed,
                );
                heights[x][y] = noise_value * params.amplitude + params.base_height;
            }
        }

        Ok(Heightmap {
            width: params.width,
            height: params.height,
            heights,
            scale: params.scale,
            min_height: params.base_height - params.amplitude,
            max_height: params.base_height + params.amplitude,
        })
    }

    fn generate_simplex_heightmap(&self, params: TerrainParams) -> RobinResult<Heightmap> {
        let generator = self.get_generator(&NoiseType::Simplex)?;
        let mut heights = vec![vec![0.0; params.height]; params.width];

        for x in 0..params.width {
            for y in 0..params.height {
                let noise_value = generator.sample_2d(
                    x as f32 * params.scale,
                    y as f32 * params.scale,
                    params.scale,
                    params.seed,
                );
                heights[x][y] = noise_value * params.amplitude + params.base_height;
            }
        }

        Ok(Heightmap {
            width: params.width,
            height: params.height,
            heights,
            scale: params.scale,
            min_height: params.base_height - params.amplitude,
            max_height: params.base_height + params.amplitude,
        })
    }

    fn generate_ridged_heightmap(&self, params: TerrainParams) -> RobinResult<Heightmap> {
        let generator = self.get_generator(&NoiseType::Ridged)?;
        let mut heights = vec![vec![0.0; params.height]; params.width];

        for x in 0..params.width {
            for y in 0..params.height {
                let noise_value = generator.sample_2d(
                    x as f32 * params.scale,
                    y as f32 * params.scale,
                    params.scale,
                    params.seed,
                );
                // Create ridged effect by using absolute value
                let ridged_value = 1.0 - noise_value.abs();
                heights[x][y] = ridged_value * params.amplitude + params.base_height;
            }
        }

        Ok(Heightmap {
            width: params.width,
            height: params.height,
            heights,
            scale: params.scale,
            min_height: params.base_height,
            max_height: params.base_height + params.amplitude,
        })
    }

    fn generate_fractal_heightmap(&self, params: TerrainParams) -> RobinResult<Heightmap> {
        let generator = self.get_generator(&NoiseType::Fractal)?;
        let mut heights = vec![vec![0.0; params.height]; params.width];

        for x in 0..params.width {
            for y in 0..params.height {
                let mut value = 0.0;
                let mut amplitude = 1.0;
                let mut frequency = params.scale;

                // Generate fractal noise with multiple octaves
                for _ in 0..6 {
                    value += generator.sample_2d(
                        x as f32 * frequency,
                        y as f32 * frequency,
                        frequency,
                        params.seed,
                    ) * amplitude;

                    amplitude *= 0.5;
                    frequency *= 2.0;
                }

                heights[x][y] = value * params.amplitude + params.base_height;
            }
        }

        Ok(Heightmap {
            width: params.width,
            height: params.height,
            heights,
            scale: params.scale,
            min_height: params.base_height - params.amplitude,
            max_height: params.base_height + params.amplitude,
        })
    }

    fn generate_voronoi_heightmap(&self, params: TerrainParams) -> RobinResult<Heightmap> {
        let generator = self.get_generator(&NoiseType::Voronoi)?;
        let mut heights = vec![vec![0.0; params.height]; params.width];

        for x in 0..params.width {
            for y in 0..params.height {
                let noise_value = generator.sample_2d(
                    x as f32 * params.scale,
                    y as f32 * params.scale,
                    params.scale,
                    params.seed,
                );
                heights[x][y] = noise_value * params.amplitude + params.base_height;
            }
        }

        Ok(Heightmap {
            width: params.width,
            height: params.height,
            heights,
            scale: params.scale,
            min_height: params.base_height - params.amplitude,
            max_height: params.base_height + params.amplitude,
        })
    }

    fn generate_turbulence_heightmap(&self, params: TerrainParams) -> RobinResult<Heightmap> {
        let generator = self.get_generator(&NoiseType::Turbulence)?;
        let mut heights = vec![vec![0.0; params.height]; params.width];

        for x in 0..params.width {
            for y in 0..params.height {
                let noise_value = generator.sample_2d(
                    x as f32 * params.scale,
                    y as f32 * params.scale,
                    params.scale,
                    params.seed,
                );
                // Apply turbulence effect
                let turbulent_value = noise_value.abs();
                heights[x][y] = turbulent_value * params.amplitude + params.base_height;
            }
        }

        Ok(Heightmap {
            width: params.width,
            height: params.height,
            heights,
            scale: params.scale,
            min_height: params.base_height,
            max_height: params.base_height + params.amplitude,
        })
    }

    fn generate_marble_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let pattern = self.patterns.get("marble_veins")
            .ok_or_else(|| RobinError::NoiseError("Marble pattern not found".to_string()))?;

        self.generate_texture_from_pattern(params, pattern)
    }

    fn generate_wood_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let pattern = self.patterns.get("wood_grain")
            .ok_or_else(|| RobinError::NoiseError("Wood pattern not found".to_string()))?;

        self.generate_texture_from_pattern(params, pattern)
    }

    fn generate_stone_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let mut texture_data = Vec::new();
        let generator = self.get_generator(&NoiseType::Perlin)?;

        for y in 0..params.height {
            for x in 0..params.width {
                let noise_value = generator.sample_2d(
                    x as f32 * 0.05,
                    y as f32 * 0.05,
                    0.05,
                    params.seed,
                );

                let base_gray = 0.6 + noise_value * 0.3;
                let r = (base_gray * 255.0) as u8;
                let g = (base_gray * 255.0) as u8;
                let b = ((base_gray + 0.1) * 255.0) as u8;

                texture_data.extend_from_slice(&[r, g, b, 255]);
            }
        }

        Ok(texture_data)
    }

    fn generate_fabric_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let mut texture_data = Vec::new();
        let generator = self.get_generator(&NoiseType::Perlin)?;

        for y in 0..params.height {
            for x in 0..params.width {
                // Create fabric weave pattern
                let weave_x = ((x as f32 * 0.2).sin() + 1.0) * 0.5;
                let weave_y = ((y as f32 * 0.2).sin() + 1.0) * 0.5;
                let weave_pattern = (weave_x + weave_y) * 0.5;

                let noise_value = generator.sample_2d(
                    x as f32 * 0.1,
                    y as f32 * 0.1,
                    0.1,
                    params.seed,
                );

                let fabric_value = weave_pattern * 0.7 + noise_value * 0.3;
                let color_intensity = (fabric_value * 255.0) as u8;

                texture_data.extend_from_slice(&[color_intensity, color_intensity, color_intensity, 255]);
            }
        }

        Ok(texture_data)
    }

    fn generate_metal_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let mut texture_data = Vec::new();
        let generator = self.get_generator(&NoiseType::Perlin)?;

        for y in 0..params.height {
            for x in 0..params.width {
                let noise_value = generator.sample_2d(
                    x as f32 * 0.02,
                    y as f32 * 0.02,
                    0.02,
                    params.seed,
                );

                // Metallic appearance with highlights
                let base_metal = 0.5 + noise_value * 0.2;
                let highlight = ((x + y) % 10) as f32 / 10.0 * 0.3;
                let metal_value = (base_metal + highlight).min(1.0);

                let r = (metal_value * 200.0) as u8;
                let g = (metal_value * 200.0) as u8;
                let b = (metal_value * 255.0) as u8;

                texture_data.extend_from_slice(&[r, g, b, 255]);
            }
        }

        Ok(texture_data)
    }

    fn generate_organic_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let mut texture_data = Vec::new();
        let generator = self.get_generator(&NoiseType::Simplex)?;

        for y in 0..params.height {
            for x in 0..params.width {
                // Multi-octave organic noise
                let mut organic_value = 0.0;
                let mut amplitude = 1.0;
                let mut frequency = 0.01;

                for _ in 0..4 {
                    organic_value += generator.sample_2d(
                        x as f32 * frequency,
                        y as f32 * frequency,
                        frequency,
                        params.seed,
                    ) * amplitude;

                    amplitude *= 0.5;
                    frequency *= 2.0;
                }

                let normalized_value = (organic_value + 1.0) * 0.5;
                let r = (normalized_value * 100.0 + 50.0) as u8;
                let g = (normalized_value * 150.0 + 80.0) as u8;
                let b = (normalized_value * 80.0 + 30.0) as u8;

                texture_data.extend_from_slice(&[r, g, b, 255]);
            }
        }

        Ok(texture_data)
    }

    fn generate_abstract_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let mut texture_data = Vec::new();
        let generator1 = self.get_generator(&NoiseType::Perlin)?;
        let generator2 = self.get_generator(&NoiseType::Simplex)?;

        for y in 0..params.height {
            for x in 0..params.width {
                let noise1 = generator1.sample_2d(x as f32 * 0.03, y as f32 * 0.03, 0.03, params.seed);
                let noise2 = generator2.sample_2d(x as f32 * 0.08, y as f32 * 0.08, 0.08, params.seed + 1000);
                let noise3 = generator1.sample_2d(x as f32 * 0.15, y as f32 * 0.15, 0.15, params.seed + 2000);

                let r = ((noise1 + 1.0) * 0.5 * 255.0) as u8;
                let g = ((noise2 + 1.0) * 0.5 * 255.0) as u8;
                let b = ((noise3 + 1.0) * 0.5 * 255.0) as u8;

                texture_data.extend_from_slice(&[r, g, b, 255]);
            }
        }

        Ok(texture_data)
    }

    fn generate_texture_from_pattern(&self, params: SurfaceParams, pattern: &NoisePattern) -> RobinResult<Vec<u8>> {
        let mut texture_data = Vec::new();

        for y in 0..params.height {
            for x in 0..params.width {
                let mut final_value = 0.0;

                // Apply all layers
                for layer in &pattern.layers {
                    let generator = self.get_generator(&layer.noise_type)?;
                    let noise_value = generator.sample_2d(
                        x as f32 * layer.frequency,
                        y as f32 * layer.frequency,
                        layer.frequency,
                        params.seed,
                    );
                    final_value += noise_value * layer.amplitude;
                }

                // Apply post-processing
                for processing in &pattern.post_processing {
                    final_value = self.apply_post_processing(final_value, processing, x as f32, y as f32);
                }

                let color_value = ((final_value + 1.0) * 0.5 * 255.0).clamp(0.0, 255.0) as u8;
                texture_data.extend_from_slice(&[color_value, color_value, color_value, 255]);
            }
        }

        Ok(texture_data)
    }

    fn generate_glass_texture(&self, params: SurfaceParams) -> RobinResult<Vec<u8>> {
        let mut texture_data = Vec::new();
        let generator = self.get_generator(&NoiseType::Perlin)?;

        for y in 0..params.height {
            for x in 0..params.width {
                // Create glass-like surface with subtle variations
                let noise_value = generator.sample_2d(
                    x as f32 * 0.05,
                    y as f32 * 0.05,
                    0.05,
                    params.seed,
                );

                // Glass has very subtle surface variations with high transparency
                let base_glass = 0.9 + noise_value * 0.1;
                let glass_value = base_glass.clamp(0.0, 1.0);

                // Light blue-tint typical of glass
                let r = (glass_value * 220.0) as u8;
                let g = (glass_value * 235.0) as u8;
                let b = (glass_value * 255.0) as u8;

                texture_data.extend_from_slice(&[r, g, b, 64]); // Low alpha for transparency
            }
        }

        Ok(texture_data)
    }

    fn generate_normal_map(&self, params: &SurfaceParams) -> RobinResult<Vec<u8>> {
        // Generate normal map from height data
        let mut normal_data = Vec::new();
        let generator = self.get_generator(&NoiseType::Perlin)?;

        for y in 0..params.height {
            for x in 0..params.width {
                // Sample neighboring pixels for gradient calculation
                let height_l = generator.sample_2d((x.saturating_sub(1)) as f32 * 0.1, y as f32 * 0.1, 0.1, params.seed);
                let height_r = generator.sample_2d((x + 1) as f32 * 0.1, y as f32 * 0.1, 0.1, params.seed);
                let height_u = generator.sample_2d(x as f32 * 0.1, (y.saturating_sub(1)) as f32 * 0.1, 0.1, params.seed);
                let height_d = generator.sample_2d(x as f32 * 0.1, (y + 1) as f32 * 0.1, 0.1, params.seed);

                let dx = height_r - height_l;
                let dy = height_d - height_u;

                // Convert gradient to normal vector
                let normal_x = ((dx + 1.0) * 0.5 * 255.0) as u8;
                let normal_y = ((dy + 1.0) * 0.5 * 255.0) as u8;
                let normal_z = 255u8; // Pointing up

                normal_data.extend_from_slice(&[normal_x, normal_y, normal_z, 255]);
            }
        }

        Ok(normal_data)
    }

    fn generate_roughness_map(&self, params: &SurfaceParams) -> RobinResult<Vec<u8>> {
        let mut roughness_data = Vec::new();
        let generator = self.get_generator(&NoiseType::Perlin)?;

        for y in 0..params.height {
            for x in 0..params.width {
                let noise_value = generator.sample_2d(
                    x as f32 * 0.05,
                    y as f32 * 0.05,
                    0.05,
                    params.seed,
                );

                let roughness = ((noise_value + 1.0) * 0.5 * 255.0) as u8;
                roughness_data.extend_from_slice(&[roughness, roughness, roughness, 255]);
            }
        }

        Ok(roughness_data)
    }

    fn get_generator(&self, noise_type: &NoiseType) -> RobinResult<&NoiseGenerator> {
        let key = match noise_type {
            NoiseType::Perlin => "perlin",
            NoiseType::Simplex => "simplex",
            NoiseType::Ridged => "ridged",
            NoiseType::Fractal => "fractal",
            NoiseType::Voronoi => "voronoi",
            NoiseType::Turbulence => "turbulence",
        };

        self.generators.get(key)
            .ok_or_else(|| RobinError::NoiseError(format!("Generator not found: {}", key)))
    }

    fn apply_distribution_bias(&self, position: Vec3, distribution_type: &DistributionType, noise_value: f32) -> Vec3 {
        match distribution_type {
            DistributionType::Uniform => position,
            DistributionType::Clustered => {
                let cluster_factor = noise_value * noise_value;
                Vec3::new(
                    position.x * (1.0 + cluster_factor * 0.5),
                    position.y,
                    position.z * (1.0 + cluster_factor * 0.5),
                )
            },
            DistributionType::Sparse => {
                let sparse_factor = (1.0 - noise_value) * 2.0;
                Vec3::new(
                    position.x * sparse_factor,
                    position.y,
                    position.z * sparse_factor,
                )
            },
            DistributionType::Linear => {
                Vec3::new(
                    position.x,
                    position.y * noise_value,
                    position.z,
                )
            },
            DistributionType::Radial => {
                let center = Vec3::new(0.5, 0.5, 0.5);
                let direction = position - center;
                center + direction * noise_value
            },
        }
    }

    fn noise_to_color(&self, noise_value: f32, palette: &ColorPalette, contrast: f32) -> Color {
        let normalized = ((noise_value + 1.0) * 0.5).clamp(0.0, 1.0);
        let contrasted = (normalized * contrast).clamp(0.0, 1.0);

        match palette {
            ColorPalette::Grayscale => Color::new(contrasted, contrasted, contrasted, 1.0),
            ColorPalette::Warm => Color::new(
                1.0,
                contrasted * 0.7 + 0.3,
                contrasted * 0.3,
                1.0,
            ),
            ColorPalette::Cool => Color::new(
                contrasted * 0.3,
                contrasted * 0.7 + 0.3,
                1.0,
                1.0,
            ),
            ColorPalette::Earth => Color::new(
                contrasted * 0.6 + 0.3,
                contrasted * 0.5 + 0.2,
                contrasted * 0.3,
                1.0,
            ),
            ColorPalette::Ocean => Color::new(
                contrasted * 0.2,
                contrasted * 0.5 + 0.3,
                contrasted * 0.8 + 0.2,
                1.0,
            ),
            ColorPalette::Fire => Color::new(
                1.0,
                contrasted * 0.8,
                contrasted * 0.2,
                1.0,
            ),
        }
    }

    fn process_volumetric_value(&self, value: f32, processing: &VolumetricProcessing) -> f32 {
        match processing {
            VolumetricProcessing::None => value,
            VolumetricProcessing::Density => (value + 1.0) * 0.5,
            VolumetricProcessing::Clouds => {
                let density = (value + 1.0) * 0.5;
                if density > 0.6 { density } else { 0.0 }
            },
            VolumetricProcessing::Smoke => {
                let density = (value + 1.0) * 0.5;
                density.powf(2.0)
            },
        }
    }

    fn apply_post_processing(&self, value: f32, processing: &PostProcessing, x: f32, y: f32) -> f32 {
        match processing {
            PostProcessing::Clamp(min, max) => value.clamp(*min, *max),
            PostProcessing::Smooth(factor) => {
                // Simple smoothing
                value * (1.0 - factor) + 0.5 * factor
            },
            PostProcessing::Sharpen(factor) => {
                value + (value - 0.5) * factor
            },
            PostProcessing::Contrast(factor) => {
                (value - 0.5) * factor + 0.5
            },
            PostProcessing::Bias(bias) => value + bias,
            PostProcessing::WoodGrain => {
                // Wood grain effect using sinusoidal rings
                let ring_value = (value * 20.0 + x * 0.1).sin();
                value + ring_value * 0.1
            },
            PostProcessing::MarbleVeins => {
                // Marble vein effect
                let vein_value = (value * 10.0 + (x + y) * 0.01).sin();
                value + vein_value * 0.3
            },
        }
    }
}

/// Noise generator implementations
#[derive(Debug)]
pub struct NoiseGenerator {
    noise_type: NoiseType,
    permutation_table: Vec<usize>,
}

impl NoiseGenerator {
    fn new(noise_type: NoiseType) -> Self {
        // Initialize permutation table for Perlin noise
        let mut permutation_table = (0..256).collect::<Vec<_>>();
        
        // Simple shuffle using system time as seed
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;

        for i in (1..permutation_table.len()).rev() {
            let j = (seed + i) % (i + 1);
            permutation_table.swap(i, j);
        }

        Self {
            noise_type,
            permutation_table,
        }
    }

    fn sample_1d(&self, x: f32, seed: u32) -> f32 {
        match self.noise_type {
            NoiseType::Perlin => self.perlin_1d(x, seed),
            NoiseType::Simplex => self.simplex_1d(x, seed),
            _ => self.perlin_1d(x, seed), // Fallback
        }
    }

    fn sample_2d(&self, x: f32, y: f32, scale: f32, seed: u32) -> f32 {
        match self.noise_type {
            NoiseType::Perlin => self.perlin_2d(x, y, seed),
            NoiseType::Simplex => self.simplex_2d(x, y, seed),
            NoiseType::Ridged => self.ridged_2d(x, y, seed),
            NoiseType::Fractal => self.fractal_2d(x, y, seed),
            NoiseType::Voronoi => self.voronoi_2d(x, y, seed),
            NoiseType::Turbulence => self.turbulence_2d(x, y, seed),
        }
    }

    fn sample_3d(&self, x: f32, y: f32, z: f32, scale: f32, seed: u32) -> f32 {
        match self.noise_type {
            NoiseType::Perlin => self.perlin_3d(x, y, z, seed),
            NoiseType::Simplex => self.simplex_3d(x, y, z, seed),
            _ => self.perlin_3d(x, y, z, seed), // Fallback
        }
    }

    // Simplified noise implementations (in a real system, you'd use a proper noise library)
    fn perlin_1d(&self, x: f32, _seed: u32) -> f32 {
        let xi = x.floor() as i32 & 255;
        let xf = x - x.floor();
        
        let fade_x = self.fade(xf);
        
        let a = self.permutation_table[xi as usize & 255] as f32 / 255.0 * 2.0 - 1.0;
        let b = self.permutation_table[(xi + 1) as usize & 255] as f32 / 255.0 * 2.0 - 1.0;
        
        self.lerp(a, b, fade_x)
    }

    fn perlin_2d(&self, x: f32, y: f32, _seed: u32) -> f32 {
        let xi = x.floor() as i32 & 255;
        let yi = y.floor() as i32 & 255;
        let xf = x - x.floor();
        let yf = y - y.floor();
        
        let fade_x = self.fade(xf);
        let fade_y = self.fade(yf);
        
        let aa = self.permutation_table[(self.permutation_table[xi as usize & 255] + yi as usize) & 255];
        let ab = self.permutation_table[(self.permutation_table[xi as usize & 255] + yi as usize + 1) & 255];
        let ba = self.permutation_table[(self.permutation_table[(xi + 1) as usize & 255] + yi as usize) & 255];
        let bb = self.permutation_table[(self.permutation_table[(xi + 1) as usize & 255] + yi as usize + 1) & 255];
        
        let grad_aa = self.grad_2d(aa, xf, yf);
        let grad_ab = self.grad_2d(ab, xf, yf - 1.0);
        let grad_ba = self.grad_2d(ba, xf - 1.0, yf);
        let grad_bb = self.grad_2d(bb, xf - 1.0, yf - 1.0);
        
        let x1 = self.lerp(grad_aa, grad_ba, fade_x);
        let x2 = self.lerp(grad_ab, grad_bb, fade_x);
        
        self.lerp(x1, x2, fade_y)
    }

    fn perlin_3d(&self, x: f32, y: f32, z: f32, _seed: u32) -> f32 {
        // Simplified 3D Perlin noise
        let noise_xy = self.perlin_2d(x, y, 0);
        let noise_xz = self.perlin_2d(x, z, 1000);
        let noise_yz = self.perlin_2d(y, z, 2000);
        
        (noise_xy + noise_xz + noise_yz) / 3.0
    }

    fn simplex_1d(&self, x: f32, _seed: u32) -> f32 {
        // Simplified simplex noise
        (x * 0.5).sin() * 0.7 + (x * 0.25).sin() * 0.3
    }

    fn simplex_2d(&self, x: f32, y: f32, _seed: u32) -> f32 {
        // Simplified simplex noise
        ((x + y) * 0.3).sin() * 0.5 + (x * 0.7).sin() * (y * 0.7).cos() * 0.5
    }

    fn simplex_3d(&self, x: f32, y: f32, z: f32, _seed: u32) -> f32 {
        // Simplified 3D simplex noise
        ((x + y + z) * 0.2).sin() * 0.33 + (x * 0.5).sin() * (y * 0.5).cos() * 0.33 + (z * 0.5).sin() * 0.33
    }

    fn ridged_2d(&self, x: f32, y: f32, seed: u32) -> f32 {
        let perlin_value = self.perlin_2d(x, y, seed);
        1.0 - perlin_value.abs()
    }

    fn fractal_2d(&self, x: f32, y: f32, seed: u32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;

        for _ in 0..4 {
            value += self.perlin_2d(x * frequency, y * frequency, seed) * amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        value
    }

    fn voronoi_2d(&self, x: f32, y: f32, _seed: u32) -> f32 {
        // Simplified Voronoi noise
        let cell_x = x.floor();
        let cell_y = y.floor();
        
        let mut min_distance = f32::INFINITY;
        
        for dx in -1..=1 {
            for dy in -1..=1 {
                let neighbor_x = cell_x + dx as f32;
                let neighbor_y = cell_y + dy as f32;
                
                // Generate point within cell
                let point_x = neighbor_x + 0.5;
                let point_y = neighbor_y + 0.5;
                
                let distance = ((x - point_x).powi(2) + (y - point_y).powi(2)).sqrt();
                min_distance = min_distance.min(distance);
            }
        }
        
        min_distance.min(1.0) * 2.0 - 1.0
    }

    fn turbulence_2d(&self, x: f32, y: f32, seed: u32) -> f32 {
        self.perlin_2d(x, y, seed).abs()
    }

    fn fade(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    fn grad_2d(&self, hash: usize, x: f32, y: f32) -> f32 {
        let h = hash & 3;
        let u = if h < 2 { x } else { y };
        let v = if h < 2 { y } else { x };
        
        let u_contrib = if (h & 1) == 0 { u } else { -u };
        let v_contrib = if (h & 2) == 0 { v } else { -v };
        
        u_contrib + v_contrib
    }
}

/// Configuration for noise system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseConfig {
    pub cache_size: usize,
    pub default_octaves: u32,
    pub default_frequency: f32,
    pub default_amplitude: f32,
    pub enable_caching: bool,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            cache_size: 100,
            default_octaves: 4,
            default_frequency: 0.01,
            default_amplitude: 1.0,
            enable_caching: true,
        }
    }
}

/// Types of noise
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Ridged,
    Fractal,
    Voronoi,
    Turbulence,
}

/// Terrain generation parameters
#[derive(Debug, Clone)]
pub struct TerrainParams {
    pub width: usize,
    pub height: usize,
    pub scale: f32,
    pub amplitude: f32,
    pub base_height: f32,
    pub noise_type: NoiseType,
    pub seed: u32,
}

/// Heightmap data structure
#[derive(Debug, Clone)]
pub struct Heightmap {
    pub width: usize,
    pub height: usize,
    pub heights: Vec<Vec<f32>>,
    pub scale: f32,
    pub min_height: f32,
    pub max_height: f32,
}

/// Surface generation parameters
#[derive(Debug, Clone)]
pub struct SurfaceParams {
    pub width: usize,
    pub height: usize,
    pub pattern: TexturePattern,
    pub seed: u32,
}

/// Generated surface data
#[derive(Debug, Clone)]
pub struct GeneratedSurface {
    pub texture_data: Vec<u8>,
    pub normal_map: Vec<u8>,
    pub roughness_map: Vec<u8>,
    pub properties: SurfaceProperties,
}

/// Surface properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceProperties {
    pub roughness: f32,
    pub metallic: f32,
    pub emissive: f32,
    pub transparency: f32,
}

impl SurfaceProperties {
    fn from_params(params: &SurfaceParams) -> Self {
        match params.pattern {
            TexturePattern::Metal => Self { roughness: 0.2, metallic: 1.0, emissive: 0.0, transparency: 0.0 },
            TexturePattern::Glass => Self { roughness: 0.0, metallic: 0.0, emissive: 0.0, transparency: 0.8 },
            TexturePattern::Wood => Self { roughness: 0.7, metallic: 0.0, emissive: 0.0, transparency: 0.0 },
            TexturePattern::Stone => Self { roughness: 0.8, metallic: 0.0, emissive: 0.0, transparency: 0.0 },
            TexturePattern::Fabric => Self { roughness: 0.9, metallic: 0.0, emissive: 0.0, transparency: 0.0 },
            TexturePattern::Marble => Self { roughness: 0.3, metallic: 0.0, emissive: 0.0, transparency: 0.0 },
            TexturePattern::Organic => Self { roughness: 0.6, metallic: 0.0, emissive: 0.0, transparency: 0.0 },
            TexturePattern::Abstract => Self { roughness: 0.5, metallic: 0.0, emissive: 0.5, transparency: 0.0 },
        }
    }
}

/// Texture patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TexturePattern {
    Marble,
    Wood,
    Stone,
    Fabric,
    Metal,
    Organic,
    Abstract,
    Glass,
}

/// Scatter distribution parameters
#[derive(Debug, Clone)]
pub struct ScatterDistributionParams {
    pub bounds: Vec3,
    pub point_count: usize,
    pub noise_type: NoiseType,
    pub scale: f32,
    pub threshold: f32,
    pub distribution_type: DistributionType,
    pub seed: u32,
}

/// Distribution types for scatter systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionType {
    Uniform,
    Clustered,
    Sparse,
    Linear,
    Radial,
}

/// Color noise parameters
#[derive(Debug, Clone)]
pub struct ColorNoiseParams {
    pub sample_count: usize,
    pub noise_type: NoiseType,
    pub frequency: f32,
    pub color_palette: ColorPalette,
    pub contrast: f32,
    pub seed: u32,
}

/// Color palettes for noise-based coloring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPalette {
    Grayscale,
    Warm,
    Cool,
    Earth,
    Ocean,
    Fire,
}

/// Volumetric noise parameters
#[derive(Debug, Clone)]
pub struct VolumetricParams {
    pub dimensions: Vec3,
    pub noise_type: NoiseType,
    pub scale: f32,
    pub processing: VolumetricProcessing,
    pub seed: u32,
}

/// Volumetric processing types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumetricProcessing {
    None,
    Density,
    Clouds,
    Smoke,
}

/// Noise pattern for layered generation
#[derive(Debug, Clone)]
pub struct NoisePattern {
    pub name: String,
    pub layers: Vec<NoiseLayer>,
    pub post_processing: Vec<PostProcessing>,
}

/// Noise layer configuration
#[derive(Debug, Clone)]
pub struct NoiseLayer {
    pub noise_type: NoiseType,
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
}

/// Post-processing effects
#[derive(Debug, Clone)]
pub enum PostProcessing {
    Clamp(f32, f32),
    Smooth(f32),
    Sharpen(f32),
    Contrast(f32),
    Bias(f32),
    WoodGrain,
    MarbleVeins,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_system_creation() {
        let config = NoiseConfig::default();
        let system = NoiseSystem::new(config);
        
        assert!(!system.generators.is_empty());
        assert!(!system.patterns.is_empty());
    }

    #[test]
    fn test_heightmap_generation() {
        let config = NoiseConfig::default();
        let mut system = NoiseSystem::new(config);
        
        let params = TerrainParams {
            width: 10,
            height: 10,
            scale: 0.1,
            amplitude: 10.0,
            base_height: 50.0,
            noise_type: NoiseType::Perlin,
            seed: 12345,
        };
        
        let heightmap = system.generate_heightmap(params);
        assert!(heightmap.is_ok());
        
        let hm = heightmap.unwrap();
        assert_eq!(hm.width, 10);
        assert_eq!(hm.height, 10);
        assert_eq!(hm.heights.len(), 10);
    }

    #[test]
    fn test_texture_generation() {
        let config = NoiseConfig::default();
        let mut system = NoiseSystem::new(config);
        
        let params = SurfaceParams {
            width: 8,
            height: 8,
            pattern: TexturePattern::Stone,
            seed: 12345,
        };
        
        let surface = system.generate_procedural_texture(params);
        assert!(surface.is_ok());
        
        let surf = surface.unwrap();
        assert_eq!(surf.texture_data.len(), 8 * 8 * 4); // RGBA
    }

    #[test]
    fn test_scatter_distribution() {
        let config = NoiseConfig::default();
        let system = NoiseSystem::new(config);
        
        let params = ScatterDistributionParams {
            bounds: Vec3::new(100.0, 100.0, 100.0),
            point_count: 100,
            noise_type: NoiseType::Perlin,
            scale: 0.1,
            threshold: 0.0,
            distribution_type: DistributionType::Uniform,
            seed: 12345,
        };
        
        let points = system.generate_scatter_distribution(params);
        assert!(points.is_ok());
        
        let pts = points.unwrap();
        assert!(!pts.is_empty());
    }
}