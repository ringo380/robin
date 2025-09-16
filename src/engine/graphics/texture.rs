use crate::engine::assets::Asset;
use wgpu::util::DeviceExt;
use image::GenericImageView;
use std::path::Path;
use std::collections::HashMap;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub format: wgpu::TextureFormat,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("size", &self.size)
            .field("format", &self.format)
            .finish()
    }
}

// Note: Clone is not implemented for Texture due to GPU resource management complexity
// Use Rc<Texture> or Arc<Texture> for shared ownership instead

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, label)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            size,
            format,
        })
    }

    pub fn create_solid_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: [u8; 4],
        size: (u32, u32),
        label: Option<&str>,
    ) -> Self {
        let (width, height) = size;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        
        for _ in 0..(width * height) {
            pixels.extend_from_slice(&color);
        }

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size: texture_size,
            format,
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn default() -> Self {
        // This is a placeholder that panics - real code should use proper constructors
        panic!("Texture::default() called - use proper Texture::new() or Texture::from_image() constructors")
    }
}

impl Asset for Texture {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        // This is a placeholder - in practice, you'd need access to device/queue
        // Real implementation would be through TextureManager
        Err("Use TextureManager::load_texture instead".into())
    }
}

pub struct TextureManager {
    textures: HashMap<String, Texture>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: HashMap<String, wgpu::BindGroup>,
}

impl TextureManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Texture Bind Group Layout"),
        });

        Self {
            textures: HashMap::new(),
            bind_group_layout,
            bind_groups: HashMap::new(),
        }
    }

    pub fn load_texture<P: AsRef<Path>>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let texture = Texture::new(device, queue, &bytes, Some(name))?;
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some(&format!("{} Bind Group", name)),
        });

        self.textures.insert(name.to_string(), texture);
        self.bind_groups.insert(name.to_string(), bind_group);
        
        log::info!("Loaded texture: {}", name);
        Ok(())
    }

    pub fn create_solid_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        color: [u8; 4],
        size: (u32, u32),
    ) {
        let texture = Texture::create_solid_color(device, queue, color, size, Some(name));
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some(&format!("{} Bind Group", name)),
        });

        self.textures.insert(name.to_string(), texture);
        self.bind_groups.insert(name.to_string(), bind_group);
        
        log::info!("Created solid texture: {} ({:?})", name, color);
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.textures.get(name)
    }

    pub fn get_bind_group(&self, name: &str) -> Option<&wgpu::BindGroup> {
        self.bind_groups.get(name)
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn create_default_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // White pixel for untextured sprites
        self.create_solid_texture(device, queue, "white", [255, 255, 255, 255], (1, 1));
        
        // Particle textures
        self.create_solid_texture(device, queue, "particle_circle", [255, 255, 255, 255], (8, 8));
        self.create_solid_texture(device, queue, "particle_square", [255, 255, 255, 255], (4, 4));
        
        // UI textures
        self.create_solid_texture(device, queue, "ui_button", [100, 100, 100, 255], (64, 32));
        self.create_solid_texture(device, queue, "ui_panel", [50, 50, 50, 200], (128, 128));
    }
}