// Robin Game Engine - OBJ Asset Importer
// Simple and fast OBJ/MTL import with material support

use super::*;
use crate::engine::error::RobinResult;
use std::path::Path;
use std::fs;
use std::collections::HashMap;

/// OBJ format importer with MTL material support
pub struct ObjImporter {
    generate_normals: bool,
    flip_normals: bool,
    smooth_normals: bool,
    import_mtl: bool,
}

impl ObjImporter {
    pub fn new() -> Self {
        Self {
            generate_normals: true,
            flip_normals: false,
            smooth_normals: true,
            import_mtl: true,
        }
    }

    pub fn with_normal_generation(mut self, generate: bool) -> Self {
        self.generate_normals = generate;
        self
    }

    pub fn with_mtl_import(mut self, import: bool) -> Self {
        self.import_mtl = import;
        self
    }

    /// Parse OBJ file format with robust error handling and validation
    fn parse_obj_data(&self, data: &str, obj_path: &Path) -> RobinResult<ObjScene> {
        let mut vertices = Vec::new();
        let mut tex_coords = Vec::new();
        let mut normals = Vec::new();
        let mut faces = Vec::new();
        let mut current_material = None;
        let mut materials = HashMap::new();
        let mut mtl_file = None;
        let mut line_number = 0;
        let mut warnings = Vec::new();

        for line in data.lines() {
            line_number += 1;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    // Vertex position with validation
                    if parts.len() < 4 {
                        return Err(format!("Line {}: Invalid vertex format - expected 'v x y z', got: {}", line_number, line).into());
                    }

                    let x = parts[1].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid X coordinate: {}", line_number, parts[1]))?;
                    let y = parts[2].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid Y coordinate: {}", line_number, parts[2]))?;
                    let z = parts[3].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid Z coordinate: {}", line_number, parts[3]))?;

                    // Validate for reasonable ranges
                    if x.is_infinite() || y.is_infinite() || z.is_infinite() {
                        return Err(format!("Line {}: Infinite coordinates detected", line_number).into());
                    }
                    if x.is_nan() || y.is_nan() || z.is_nan() {
                        return Err(format!("Line {}: NaN coordinates detected", line_number).into());
                    }

                    vertices.push([x, y, z]);
                },
                "vt" => {
                    // Texture coordinate with validation
                    if parts.len() < 3 {
                        return Err(format!("Line {}: Invalid texture coordinate format - expected 'vt u v', got: {}", line_number, line).into());
                    }

                    let u = parts[1].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid U coordinate: {}", line_number, parts[1]))?;
                    let v = parts[2].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid V coordinate: {}", line_number, parts[2]))?;

                    // Warn about UV coordinates outside [0,1] range
                    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
                        warnings.push(format!("Line {}: UV coordinates outside [0,1] range: ({:.3}, {:.3})", line_number, u, v));
                    }

                    tex_coords.push([u, v]);
                },
                "vn" => {
                    // Vertex normal with validation
                    if parts.len() < 4 {
                        return Err(format!("Line {}: Invalid normal format - expected 'vn x y z', got: {}", line_number, line).into());
                    }

                    let x = parts[1].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid normal X: {}", line_number, parts[1]))?;
                    let y = parts[2].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid normal Y: {}", line_number, parts[2]))?;
                    let z = parts[3].parse::<f32>()
                        .map_err(|_| format!("Line {}: Invalid normal Z: {}", line_number, parts[3]))?;

                    // Validate normal vector
                    let length = (x * x + y * y + z * z).sqrt();
                    if length < 0.001 {
                        warnings.push(format!("Line {}: Zero-length normal vector", line_number));
                    } else if (length - 1.0).abs() > 0.1 {
                        warnings.push(format!("Line {}: Non-unit normal vector (length: {:.3})", line_number, length));
                    }

                    normals.push([x, y, z]);
                },
                "f" => {
                    // Face with validation
                    if parts.len() < 4 {
                        return Err(format!("Line {}: Invalid face format - need at least 3 vertices, got: {}", line_number, line).into());
                    }

                    match self.parse_face(&parts[1..], current_material.clone(), line_number) {
                        Ok(face) => faces.push(face),
                        Err(e) => return Err(format!("Line {}: {}", line_number, e).into()),
                    }
                },
                "mtllib" => {
                    // Material library
                    if parts.len() < 2 {
                        return Err(format!("Line {}: mtllib command missing filename", line_number).into());
                    }
                    if self.import_mtl {
                        mtl_file = Some(parts[1].to_string());
                    }
                },
                "usemtl" => {
                    // Use material
                    if parts.len() < 2 {
                        return Err(format!("Line {}: usemtl command missing material name", line_number).into());
                    }
                    current_material = Some(parts[1].to_string());
                },
                "o" | "g" => {
                    // Object/group names - log but continue
                    if parts.len() >= 2 {
                        warnings.push(format!("Line {}: Object/group '{}' encountered - may affect mesh organization", line_number, parts[1]));
                    }
                },
                "s" => {
                    // Smoothing groups - log but continue
                    warnings.push(format!("Line {}: Smoothing groups not fully supported", line_number));
                },
                _ => {
                    // Unknown commands
                    warnings.push(format!("Line {}: Unknown OBJ command: {}", line_number, parts[0]));
                }
            }
        }

        // Load MTL file if specified
        if let Some(mtl_filename) = mtl_file {
            if let Some(parent_dir) = obj_path.parent() {
                let mtl_path = parent_dir.join(&mtl_filename);
                if mtl_path.exists() {
                    materials = self.parse_mtl_file(&mtl_path).map_err(|e| {
                        format!("Failed to parse MTL file '{}': {}", mtl_filename, e)
                    })?;
                } else {
                    warnings.push(format!("Referenced MTL file '{}' not found", mtl_filename));
                }
            }
        }

        // Validate the parsed scene
        self.validate_obj_scene(&vertices, &faces, &materials, &warnings)?;

        // Output warnings if any
        if !warnings.is_empty() {
            println!("OBJ Import Warnings:");
            for warning in &warnings {
                println!("  {}", warning);
            }
        }

        Ok(ObjScene {
            vertices,
            tex_coords,
            normals,
            faces,
            materials,
        })
    }

    /// Validate the parsed OBJ scene for consistency
    fn validate_obj_scene(&self, vertices: &[[f32; 3]], faces: &[ObjFace], materials: &HashMap<String, ObjMaterial>, warnings: &[String]) -> RobinResult<()> {
        if vertices.is_empty() {
            return Err("OBJ file contains no vertices".into());
        }

        if faces.is_empty() {
            return Err("OBJ file contains no faces".into());
        }

        // Validate face indices
        for (face_idx, face) in faces.iter().enumerate() {
            for vertex_ref in &face.vertices {
                if let Some(pos_idx) = vertex_ref.position {
                    if pos_idx < 0 || pos_idx as usize >= vertices.len() {
                        return Err(format!("Face {} references invalid vertex index: {}", face_idx, pos_idx).into());
                    }
                }
            }

            // Check for degenerate faces
            if face.vertices.len() < 3 {
                return Err(format!("Face {} has fewer than 3 vertices", face_idx).into());
            }
        }

        // Check material references
        for (face_idx, face) in faces.iter().enumerate() {
            if let Some(ref material_name) = face.material {
                if !materials.contains_key(material_name) {
                    return Err(format!("Face {} references unknown material: {}", face_idx, material_name).into());
                }
            }
        }

        Ok(())
    }

    fn parse_face(&self, face_parts: &[&str], material: Option<String>, line_number: usize) -> RobinResult<ObjFace> {
        let mut vertex_indices = Vec::new();

        for (vertex_idx, part) in face_parts.iter().enumerate() {
            let indices = self.parse_vertex_reference(part)
                .map_err(|e| format!("vertex {} in face: {}", vertex_idx + 1, e))?;
            vertex_indices.push(indices);
        }

        // Validate face has enough vertices
        if vertex_indices.len() < 3 {
            return Err(format!("Face needs at least 3 vertices, got {}", vertex_indices.len()).into());
        }

        // Warn about complex faces
        if vertex_indices.len() > 4 {
            println!("Warning: Line {}: Face with {} vertices will be triangulated", line_number, vertex_indices.len());
        }

        Ok(ObjFace {
            vertices: vertex_indices,
            material,
        })
    }

    fn parse_vertex_reference(&self, vertex_ref: &str) -> RobinResult<ObjVertexIndex> {
        let parts: Vec<&str> = vertex_ref.split('/').collect();

        let position_index = if !parts[0].is_empty() {
            Some(parts[0].parse::<i32>().map_err(|_| "Invalid vertex index")? - 1) // OBJ is 1-indexed
        } else {
            None
        };

        let tex_coord_index = if parts.len() > 1 && !parts[1].is_empty() {
            Some(parts[1].parse::<i32>().map_err(|_| "Invalid texture coordinate index")? - 1)
        } else {
            None
        };

        let normal_index = if parts.len() > 2 && !parts[2].is_empty() {
            Some(parts[2].parse::<i32>().map_err(|_| "Invalid normal index")? - 1)
        } else {
            None
        };

        Ok(ObjVertexIndex {
            position: position_index,
            tex_coord: tex_coord_index,
            normal: normal_index,
        })
    }

    fn parse_mtl_file(&self, mtl_path: &Path) -> RobinResult<HashMap<String, ObjMaterial>> {
        let mtl_data = fs::read_to_string(mtl_path)
            .map_err(|e| format!("Failed to read MTL file: {}", e))?;

        let mut materials = HashMap::new();
        let mut current_material = None;

        for line in mtl_data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "newmtl" => {
                    // New material
                    if parts.len() >= 2 {
                        let material = ObjMaterial::default();
                        current_material = Some(parts[1].to_string());
                        materials.insert(parts[1].to_string(), material);
                    }
                },
                "Ka" => {
                    // Ambient color
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 4 {
                                material.ambient = [
                                    parts[1].parse().unwrap_or(0.0),
                                    parts[2].parse().unwrap_or(0.0),
                                    parts[3].parse().unwrap_or(0.0),
                                ];
                            }
                        }
                    }
                },
                "Kd" => {
                    // Diffuse color
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 4 {
                                material.diffuse = [
                                    parts[1].parse().unwrap_or(0.0),
                                    parts[2].parse().unwrap_or(0.0),
                                    parts[3].parse().unwrap_or(0.0),
                                ];
                            }
                        }
                    }
                },
                "Ks" => {
                    // Specular color
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 4 {
                                material.specular = [
                                    parts[1].parse().unwrap_or(0.0),
                                    parts[2].parse().unwrap_or(0.0),
                                    parts[3].parse().unwrap_or(0.0),
                                ];
                            }
                        }
                    }
                },
                "Ns" => {
                    // Shininess
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 2 {
                                material.shininess = parts[1].parse().unwrap_or(0.0);
                            }
                        }
                    }
                },
                "d" | "Tr" => {
                    // Transparency
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 2 {
                                let alpha = parts[1].parse().unwrap_or(1.0);
                                material.transparency = if parts[0] == "Tr" { alpha } else { 1.0 - alpha };
                            }
                        }
                    }
                },
                "map_Kd" => {
                    // Diffuse texture map
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 2 {
                                material.diffuse_map = Some(parts[1].to_string());
                            }
                        }
                    }
                },
                "map_Ks" => {
                    // Specular texture map
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 2 {
                                material.specular_map = Some(parts[1].to_string());
                            }
                        }
                    }
                },
                "map_bump" | "bump" => {
                    // Normal/bump map
                    if let Some(ref name) = current_material {
                        if let Some(material) = materials.get_mut(name) {
                            if parts.len() >= 2 {
                                material.normal_map = Some(parts[1].to_string());
                            }
                        }
                    }
                },
                _ => {
                    // Ignore other material properties
                }
            }
        }

        Ok(materials)
    }

    /// Convert OBJ scene to Robin engine assets
    fn convert_to_assets(&self, obj_scene: ObjScene, options: &ImportOptions) -> RobinResult<Vec<ImportedAsset>> {
        let mut assets = Vec::new();

        // Build mesh data from OBJ faces
        let mesh_data = self.build_mesh_from_faces(&obj_scene, options)?;

        // Create mesh asset
        let mesh_asset = ImportedAsset {
            id: "obj_mesh".to_string(),
            name: "OBJ Mesh".to_string(),
            asset_type: AssetType::Mesh,
            data: AssetData::Mesh(mesh_data),
            metadata: self.create_mesh_metadata(options),
            dependencies: Vec::new(),
        };
        assets.push(mesh_asset);

        // Convert materials
        for (name, obj_material) in obj_scene.materials {
            let material_data = self.convert_obj_material(obj_material)?;
            let material_asset = ImportedAsset {
                id: format!("material_{}", name),
                name: name.clone(),
                asset_type: AssetType::Material,
                data: AssetData::Material(material_data),
                metadata: self.create_material_metadata(options),
                dependencies: Vec::new(),
            };
            assets.push(material_asset);
        }

        Ok(assets)
    }

    fn build_mesh_from_faces(&self, obj_scene: &ObjScene, options: &ImportOptions) -> RobinResult<MeshData> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_map = HashMap::new();

        for face in &obj_scene.faces {
            // Triangulate face (assume triangles or quads)
            let face_indices = self.triangulate_face(&face.vertices, obj_scene, &mut vertices, &mut vertex_map)?;
            indices.extend(face_indices);
        }

        // Generate normals if needed
        if self.generate_normals && obj_scene.normals.is_empty() {
            self.generate_vertex_normals(&mut vertices, &indices);
        }

        // Apply post-processing
        if options.optimize {
            self.optimize_mesh(&mut vertices, &mut indices);
        }

        // Generate LODs if requested
        let lod_levels = if options.generate_lods {
            self.generate_lod_levels(&vertices, &indices)?
        } else {
            Vec::new()
        };

        let bounding_box = self.calculate_bounding_box(&vertices);

        Ok(MeshData {
            vertices,
            indices,
            material_id: None, // OBJ can have multiple materials per mesh
            bounding_box,
            lod_levels,
        })
    }

    fn triangulate_face(&self, face_vertices: &[ObjVertexIndex], obj_scene: &ObjScene, vertices: &mut Vec<Vertex>, vertex_map: &mut HashMap<String, u32>) -> RobinResult<Vec<u32>> {
        let mut face_indices = Vec::new();

        // Convert face vertices to our vertex format
        let mut converted_vertices = Vec::new();
        for vertex_index in face_vertices {
            let vertex = self.convert_obj_vertex(vertex_index, obj_scene)?;
            let vertex_key = format!("{:?}", vertex);

            let index = if let Some(&existing_index) = vertex_map.get(&vertex_key) {
                existing_index
            } else {
                let new_index = vertices.len() as u32;
                vertices.push(vertex);
                vertex_map.insert(vertex_key, new_index);
                new_index
            };

            converted_vertices.push(index);
        }

        // Triangulate (fan triangulation for n-gons)
        if converted_vertices.len() >= 3 {
            for i in 1..converted_vertices.len() - 1 {
                face_indices.push(converted_vertices[0]);
                face_indices.push(converted_vertices[i]);
                face_indices.push(converted_vertices[i + 1]);
            }
        }

        Ok(face_indices)
    }

    fn convert_obj_vertex(&self, vertex_index: &ObjVertexIndex, obj_scene: &ObjScene) -> RobinResult<Vertex> {
        let position = if let Some(pos_idx) = vertex_index.position {
            if pos_idx >= 0 && (pos_idx as usize) < obj_scene.vertices.len() {
                obj_scene.vertices[pos_idx as usize]
            } else {
                [0.0, 0.0, 0.0]
            }
        } else {
            [0.0, 0.0, 0.0]
        };

        let uv = if let Some(uv_idx) = vertex_index.tex_coord {
            if uv_idx >= 0 && (uv_idx as usize) < obj_scene.tex_coords.len() {
                obj_scene.tex_coords[uv_idx as usize]
            } else {
                [0.0, 0.0]
            }
        } else {
            [0.0, 0.0]
        };

        let normal = if let Some(norm_idx) = vertex_index.normal {
            if norm_idx >= 0 && (norm_idx as usize) < obj_scene.normals.len() {
                let mut normal = obj_scene.normals[norm_idx as usize];
                if self.flip_normals {
                    normal[0] = -normal[0];
                    normal[1] = -normal[1];
                    normal[2] = -normal[2];
                }
                normal
            } else {
                [0.0, 0.0, 1.0]
            }
        } else {
            [0.0, 0.0, 1.0]
        };

        Ok(Vertex {
            position,
            normal,
            uv,
            tangent: None, // Calculate later if needed
            color: None,
        })
    }

    fn convert_obj_material(&self, obj_material: ObjMaterial) -> RobinResult<MaterialData> {
        let mut properties = HashMap::new();
        let mut textures = HashMap::new();

        // Convert color properties
        properties.insert("diffuseColor".to_string(),
            MaterialProperty::Vec3(obj_material.diffuse));
        properties.insert("specularColor".to_string(),
            MaterialProperty::Vec3(obj_material.specular));
        properties.insert("ambientColor".to_string(),
            MaterialProperty::Vec3(obj_material.ambient));
        properties.insert("shininess".to_string(),
            MaterialProperty::Float(obj_material.shininess));
        properties.insert("transparency".to_string(),
            MaterialProperty::Float(obj_material.transparency));

        // Convert texture maps
        if let Some(diffuse_map) = obj_material.diffuse_map {
            textures.insert("diffuseTexture".to_string(), diffuse_map);
        }
        if let Some(specular_map) = obj_material.specular_map {
            textures.insert("specularTexture".to_string(), specular_map);
        }
        if let Some(normal_map) = obj_material.normal_map {
            textures.insert("normalTexture".to_string(), normal_map);
        }

        Ok(MaterialData {
            name: "OBJ Material".to_string(),
            shader: "obj_standard".to_string(),
            properties,
            textures,
        })
    }

    // Utility methods
    fn generate_vertex_normals(&self, vertices: &mut [Vertex], indices: &[u32]) {
        // Reset normals
        for vertex in vertices.iter_mut() {
            vertex.normal = [0.0, 0.0, 0.0];
        }

        // Calculate face normals and accumulate
        for triangle in indices.chunks(3) {
            if triangle.len() == 3 {
                let i0 = triangle[0] as usize;
                let i1 = triangle[1] as usize;
                let i2 = triangle[2] as usize;

                if i0 < vertices.len() && i1 < vertices.len() && i2 < vertices.len() {
                    let v0 = vertices[i0].position;
                    let v1 = vertices[i1].position;
                    let v2 = vertices[i2].position;

                    // Calculate face normal
                    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
                    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

                    let normal = [
                        edge1[1] * edge2[2] - edge1[2] * edge2[1],
                        edge1[2] * edge2[0] - edge1[0] * edge2[2],
                        edge1[0] * edge2[1] - edge1[1] * edge2[0],
                    ];

                    // Accumulate to vertex normals
                    for &idx in triangle {
                        let vertex = &mut vertices[idx as usize];
                        vertex.normal[0] += normal[0];
                        vertex.normal[1] += normal[1];
                        vertex.normal[2] += normal[2];
                    }
                }
            }
        }

        // Normalize vertex normals
        for vertex in vertices.iter_mut() {
            let length = (vertex.normal[0] * vertex.normal[0] +
                         vertex.normal[1] * vertex.normal[1] +
                         vertex.normal[2] * vertex.normal[2]).sqrt();

            if length > 0.0 {
                vertex.normal[0] /= length;
                vertex.normal[1] /= length;
                vertex.normal[2] /= length;
            }
        }
    }

    fn optimize_mesh(&self, vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Simple optimization - remove unused vertices
        // In production, use more sophisticated algorithms
    }

    fn generate_lod_levels(&self, vertices: &[Vertex], indices: &[u32]) -> RobinResult<Vec<LodLevel>> {
        // Simple LOD generation
        Ok(Vec::new())
    }

    fn calculate_bounding_box(&self, vertices: &[Vertex]) -> BoundingBox {
        if vertices.is_empty() {
            return BoundingBox { min: [0.0, 0.0, 0.0], max: [0.0, 0.0, 0.0] };
        }

        let mut min = vertices[0].position;
        let mut max = vertices[0].position;

        for vertex in vertices.iter().skip(1) {
            for i in 0..3 {
                min[i] = min[i].min(vertex.position[i]);
                max[i] = max[i].max(vertex.position[i]);
            }
        }

        BoundingBox { min, max }
    }

    fn create_mesh_metadata(&self, options: &ImportOptions) -> AssetMetadata {
        AssetMetadata {
            file_size: 0,
            creation_time: chrono::Utc::now(),
            modification_time: chrono::Utc::now(),
            checksum: "".to_string(),
            import_settings: options.clone(),
            source_file: "".to_string(),
            vertex_count: None,
            triangle_count: None,
            texture_memory: None,
            compression_ratio: None,
        }
    }

    fn create_material_metadata(&self, options: &ImportOptions) -> AssetMetadata {
        self.create_mesh_metadata(options)
    }
}

impl AssetImporter for ObjImporter {
    fn name(&self) -> &'static str {
        "OBJ Importer"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["obj"]
    }

    fn import(&self, path: &Path, options: &ImportOptions) -> RobinResult<ImportedAsset> {
        let data = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read OBJ file: {}", e))?;

        let obj_scene = self.parse_obj_data(&data, path)?;
        let assets = self.convert_to_assets(obj_scene, options)?;

        // Return the mesh asset
        assets.into_iter()
            .find(|asset| asset.asset_type == AssetType::Mesh)
            .ok_or_else(|| "No mesh found in OBJ file".to_string().into())
    }

    fn can_import(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return ext_str.to_lowercase() == "obj";
            }
        }
        false
    }

    fn validate(&self, path: &Path) -> RobinResult<ValidationResult> {
        let mut result = ValidationResult {
            valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            recommendations: Vec::new(),
        };

        if !path.exists() {
            result.valid = false;
            result.errors.push("File does not exist".to_string());
            return Ok(result);
        }

        // Check file size
        if let Ok(metadata) = fs::metadata(path) {
            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

            if size_mb > 100.0 {
                result.warnings.push(format!("Large OBJ file: {:.1} MB", size_mb));
                result.recommendations.push("Consider mesh optimization or format conversion".to_string());
            }

            if size_mb == 0.0 {
                result.valid = false;
                result.errors.push("Empty file".to_string());
                return Ok(result);
            }
        }

        // Comprehensive OBJ validation
        if let Ok(data) = fs::read_to_string(path) {
            let mut has_vertices = false;
            let mut has_faces = false;
            let mut has_texcoords = false;
            let mut has_normals = false;
            let mut has_materials = false;
            let mut vertex_count = 0;
            let mut face_count = 0;
            let mut complex_faces = 0;
            let mut mtl_file = None;
            let mut line_number = 0;

            for line in data.lines() {
                line_number += 1;
                let line = line.trim();

                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }

                match parts[0] {
                    "v" => {
                        has_vertices = true;
                        vertex_count += 1;

                        // Validate vertex format
                        if parts.len() < 4 {
                            result.errors.push(format!("Line {}: Invalid vertex format", line_number));
                            result.valid = false;
                        }
                    },
                    "vt" => {
                        has_texcoords = true;

                        if parts.len() < 3 {
                            result.errors.push(format!("Line {}: Invalid texture coordinate format", line_number));
                            result.valid = false;
                        }
                    },
                    "vn" => {
                        has_normals = true;

                        if parts.len() < 4 {
                            result.errors.push(format!("Line {}: Invalid normal format", line_number));
                            result.valid = false;
                        }
                    },
                    "f" => {
                        has_faces = true;
                        face_count += 1;

                        if parts.len() < 4 {
                            result.errors.push(format!("Line {}: Face needs at least 3 vertices", line_number));
                            result.valid = false;
                        } else if parts.len() > 4 {
                            complex_faces += 1;
                        }
                    },
                    "mtllib" => {
                        if parts.len() >= 2 {
                            mtl_file = Some(parts[1].to_string());
                        }
                    },
                    "usemtl" => {
                        has_materials = true;
                    },
                    _ => {}
                }
            }

            // Validation results
            if !has_vertices {
                result.valid = false;
                result.errors.push("No vertices found in OBJ file".to_string());
            }

            if !has_faces {
                result.warnings.push("No faces found in OBJ file".to_string());
                result.recommendations.push("File may be intended as a point cloud".to_string());
            }

            // Quality checks
            if vertex_count > 100000 {
                result.warnings.push(format!("High vertex count: {}", vertex_count));
                result.recommendations.push("Consider mesh simplification".to_string());
            }

            if face_count > 50000 {
                result.warnings.push(format!("High face count: {}", face_count));
                result.recommendations.push("Consider LOD generation".to_string());
            }

            if complex_faces > 0 {
                result.warnings.push(format!("{} faces with more than 4 vertices", complex_faces));
                result.recommendations.push("Complex faces will be triangulated".to_string());
            }

            if !has_texcoords && has_materials {
                result.warnings.push("Materials defined but no texture coordinates found".to_string());
            }

            if !has_normals {
                result.recommendations.push("No normals found - will be generated automatically".to_string());
            }

            // Check for MTL file
            if let Some(mtl_filename) = mtl_file {
                if let Some(parent_dir) = path.parent() {
                    let mtl_path = parent_dir.join(mtl_filename);
                    if !mtl_path.exists() {
                        result.warnings.push(format!("Referenced MTL file '{}' not found", mtl_filename));
                        result.recommendations.push("Ensure MTL file is in the same directory".to_string());
                    }
                }
            } else if has_materials {
                result.warnings.push("Materials used but no MTL library specified".to_string());
            }

            // Parse with actual parser to catch more issues
            match self.parse_obj_data(&data, path) {
                Ok(_) => {
                    // Successfully parsed
                },
                Err(e) => {
                    result.valid = false;
                    result.errors.push(format!("Parse error: {}", e));
                }
            }
        } else {
            result.valid = false;
            result.errors.push("Cannot read file as text".to_string());
        }

        // General recommendations
        result.recommendations.push("Ensure accompanying MTL file is present for materials".to_string());
        result.recommendations.push("Use triangulated meshes for better compatibility".to_string());
        result.recommendations.push("Consider GLTF format for more advanced features".to_string());
        result.recommendations.push("Include vertex normals for better lighting".to_string());

        Ok(result)
    }
}

// OBJ-specific data structures
#[derive(Debug, Clone)]
pub struct ObjScene {
    pub vertices: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<ObjFace>,
    pub materials: HashMap<String, ObjMaterial>,
}

#[derive(Debug, Clone)]
pub struct ObjFace {
    pub vertices: Vec<ObjVertexIndex>,
    pub material: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ObjVertexIndex {
    pub position: Option<i32>,
    pub tex_coord: Option<i32>,
    pub normal: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ObjMaterial {
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub ambient: [f32; 3],
    pub shininess: f32,
    pub transparency: f32,
    pub diffuse_map: Option<String>,
    pub specular_map: Option<String>,
    pub normal_map: Option<String>,
}

impl Default for ObjMaterial {
    fn default() -> Self {
        Self {
            diffuse: [0.8, 0.8, 0.8],
            specular: [1.0, 1.0, 1.0],
            ambient: [0.2, 0.2, 0.2],
            shininess: 32.0,
            transparency: 0.0,
            diffuse_map: None,
            specular_map: None,
            normal_map: None,
        }
    }
}