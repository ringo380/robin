use nalgebra::{Vector3, Vector4, Point3, Matrix4, UnitQuaternion, Perspective3};
use super::{CharacterState, MovementMode};

#[derive(Clone, Debug)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
    BuildMode,
    FreeCam,
    Orbit,
}

pub struct CameraController {
    // Camera modes
    current_mode: CameraMode,
    
    // First person settings
    fov: f32,
    near_plane: f32,
    far_plane: f32,
    
    // Third person settings
    third_person_distance: f32,
    third_person_height: f32,
    orbit_speed: f32,
    zoom_speed: f32,
    min_distance: f32,
    max_distance: f32,
    
    // Build mode settings
    build_camera_speed: f32,
    build_zoom_sensitivity: f32,
    
    // Camera state
    position: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    pitch: f32,
    yaw: f32,
    
    // Smoothing
    position_smoothing: f32,
    rotation_smoothing: f32,
    target_position: Point3<f32>,
    target_pitch: f32,
    target_yaw: f32,
    
    // Projection
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    aspect_ratio: f32,
    viewport_width: f32,
    viewport_height: f32,
    
    // Build mode camera
    build_orbit_distance: f32,
    build_orbit_height: f32,
    build_focus_point: Point3<f32>,
}

impl CameraController {
    pub fn new() -> Self {
        let mut controller = Self {
            current_mode: CameraMode::FirstPerson,
            
            fov: 75.0_f32.to_radians(),
            near_plane: 0.1,
            far_plane: 1000.0,
            
            third_person_distance: 5.0,
            third_person_height: 2.0,
            orbit_speed: 2.0,
            zoom_speed: 2.0,
            min_distance: 1.0,
            max_distance: 20.0,
            
            build_camera_speed: 10.0,
            build_zoom_sensitivity: 0.5,
            
            position: Point3::new(0.0, 2.0, 0.0),
            target: Point3::new(0.0, 2.0, -1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            
            position_smoothing: 8.0,
            rotation_smoothing: 10.0,
            target_position: Point3::new(0.0, 2.0, 0.0),
            target_pitch: 0.0,
            target_yaw: 0.0,
            
            projection_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            aspect_ratio: 16.0 / 9.0,
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            
            build_orbit_distance: 10.0,
            build_orbit_height: 5.0,
            build_focus_point: Point3::new(0.0, 0.0, 0.0),
        };
        
        controller.update_projection_matrix();
        controller
    }

    pub fn update(&mut self, delta_time: f32) {
        self.smooth_camera_movement(delta_time);
        self.update_view_matrix();
    }

    pub fn update_from_character(&mut self, character_state: &CharacterState, mouse_delta: Vector3<f32>, scroll_delta: f32) {
        match self.current_mode {
            CameraMode::FirstPerson => self.update_first_person(character_state, mouse_delta),
            CameraMode::ThirdPerson => self.update_third_person(character_state, mouse_delta, scroll_delta),
            CameraMode::BuildMode => self.update_build_mode(character_state, mouse_delta, scroll_delta),
            CameraMode::FreeCam => self.update_free_cam(mouse_delta, scroll_delta),
            CameraMode::Orbit => self.update_orbit_mode(character_state, mouse_delta, scroll_delta),
        }
    }

    fn update_first_person(&mut self, character_state: &CharacterState, mouse_delta: Vector3<f32>) {
        // Camera follows character position exactly
        self.target_position = character_state.position + Vector3::new(0.0, 1.8, 0.0); // Eye level
        
        // Direct mouse look
        self.target_yaw += mouse_delta.x * 0.002;
        self.target_pitch += mouse_delta.y * 0.002;
        
        // Clamp pitch
        self.target_pitch = self.target_pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
        
        // Calculate look direction
        let look_direction = Vector3::new(
            self.target_yaw.cos() * self.target_pitch.cos(),
            self.target_pitch.sin(),
            self.target_yaw.sin() * self.target_pitch.cos(),
        );
        
        self.target = self.target_position + look_direction;
    }

    fn update_third_person(&mut self, character_state: &CharacterState, mouse_delta: Vector3<f32>, scroll_delta: f32) {
        // Orbit around character
        self.target_yaw += mouse_delta.x * 0.005;
        self.target_pitch += mouse_delta.y * 0.005;
        
        // Clamp pitch for third person
        self.target_pitch = self.target_pitch.clamp(-0.8, 0.8);
        
        // Handle zoom
        self.third_person_distance -= scroll_delta * self.zoom_speed;
        self.third_person_distance = self.third_person_distance.clamp(self.min_distance, self.max_distance);
        
        // Calculate camera position
        let character_center = character_state.position + Vector3::new(0.0, self.third_person_height, 0.0);
        
        let offset = Vector3::new(
            self.target_yaw.cos() * self.target_pitch.cos() * self.third_person_distance,
            self.target_pitch.sin() * self.third_person_distance,
            self.target_yaw.sin() * self.target_pitch.cos() * self.third_person_distance,
        );
        
        self.target_position = character_center + offset;
        self.target = character_center;
    }

    fn update_build_mode(&mut self, character_state: &CharacterState, mouse_delta: Vector3<f32>, scroll_delta: f32) {
        if character_state.build_mode {
            // In build mode, camera can move more freely but focuses on build area
            
            // Handle orbit around build focus point
            self.target_yaw += mouse_delta.x * 0.003;
            self.target_pitch += mouse_delta.y * 0.003;
            
            // Clamp pitch
            self.target_pitch = self.target_pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.2, std::f32::consts::FRAC_PI_2 - 0.2);
            
            // Handle zoom
            self.build_orbit_distance -= scroll_delta * self.build_zoom_sensitivity;
            self.build_orbit_distance = self.build_orbit_distance.clamp(2.0, 50.0);
            
            // Update build focus point based on character's look direction
            let character_forward = character_state.rotation * Vector3::new(0.0, 0.0, -1.0);
            self.build_focus_point = character_state.position + character_forward * 5.0;
            
            // Calculate camera position around build focus
            let offset = Vector3::new(
                self.target_yaw.cos() * self.target_pitch.cos() * self.build_orbit_distance,
                self.target_pitch.sin() * self.build_orbit_distance + self.build_orbit_height,
                self.target_yaw.sin() * self.target_pitch.cos() * self.build_orbit_distance,
            );
            
            self.target_position = self.build_focus_point + offset;
            self.target = self.build_focus_point;
        } else {
            // Fall back to first person when not in build mode
            self.update_first_person(character_state, mouse_delta);
        }
    }

    fn update_free_cam(&mut self, mouse_delta: Vector3<f32>, scroll_delta: f32) {
        // Free camera movement
        self.target_yaw += mouse_delta.x * 0.002;
        self.target_pitch += mouse_delta.y * 0.002;
        
        // Clamp pitch
        self.target_pitch = self.target_pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
        
        // Calculate look direction
        let look_direction = Vector3::new(
            self.target_yaw.cos() * self.target_pitch.cos(),
            self.target_pitch.sin(),
            self.target_yaw.sin() * self.target_pitch.cos(),
        );
        
        self.target = self.target_position + look_direction;
    }

    fn update_orbit_mode(&mut self, character_state: &CharacterState, mouse_delta: Vector3<f32>, scroll_delta: f32) {
        // Similar to third person but with different controls
        self.update_third_person(character_state, mouse_delta, scroll_delta);
    }

    fn smooth_camera_movement(&mut self, delta_time: f32) {
        // Smoothly interpolate position
        let position_diff = self.target_position - self.position;
        self.position += position_diff * self.position_smoothing * delta_time;
        
        // Smoothly interpolate rotation
        let pitch_diff = self.target_pitch - self.pitch;
        let yaw_diff = self.target_yaw - self.yaw;
        
        self.pitch += pitch_diff * self.rotation_smoothing * delta_time;
        self.yaw += yaw_diff * self.rotation_smoothing * delta_time;
    }

    fn update_view_matrix(&mut self) {
        // Calculate view matrix
        let eye = self.position;
        let center = self.target;
        let up = self.up;
        
        // Look-at matrix calculation
        let f = (center - eye).normalize();
        let s = f.cross(&up).normalize();
        let u = s.cross(&f);
        
        self.view_matrix = Matrix4::new(
            s.x, u.x, -f.x, 0.0,
            s.y, u.y, -f.y, 0.0,
            s.z, u.z, -f.z, 0.0,
            -s.dot(&eye.coords), -u.dot(&eye.coords), f.dot(&eye.coords), 1.0,
        );
    }

    fn update_projection_matrix(&mut self) {
        let perspective = Perspective3::new(self.aspect_ratio, self.fov, self.near_plane, self.far_plane);
        self.projection_matrix = perspective.into_inner();
    }

    // Camera mode switching
    pub fn set_camera_mode(&mut self, mode: CameraMode) {
        self.current_mode = mode;
    }

    pub fn cycle_camera_mode(&mut self) {
        self.current_mode = match self.current_mode {
            CameraMode::FirstPerson => CameraMode::ThirdPerson,
            CameraMode::ThirdPerson => CameraMode::BuildMode,
            CameraMode::BuildMode => CameraMode::FreeCam,
            CameraMode::FreeCam => CameraMode::Orbit,
            CameraMode::Orbit => CameraMode::FirstPerson,
        };
    }

    // Viewport management
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.aspect_ratio = width / height;
        self.update_projection_matrix();
    }

    pub fn set_fov(&mut self, fov_degrees: f32) {
        self.fov = fov_degrees.to_radians();
        self.update_projection_matrix();
    }

    // World to screen conversion
    pub fn world_to_screen(&self, world_pos: Point3<f32>) -> Option<(f32, f32)> {
        let view_proj = self.projection_matrix * self.view_matrix;
        let clip_pos = view_proj * world_pos.to_homogeneous();
        
        if clip_pos.w <= 0.0 {
            return None; // Behind camera
        }
        
        let ndc = clip_pos.xyz() / clip_pos.w;
        
        if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 || ndc.z < -1.0 || ndc.z > 1.0 {
            return None; // Outside view frustum
        }
        
        let screen_x = (ndc.x + 1.0) * 0.5 * self.viewport_width;
        let screen_y = (1.0 - ndc.y) * 0.5 * self.viewport_height;
        
        Some((screen_x, screen_y))
    }

    pub fn screen_to_ray(&self, screen_x: f32, screen_y: f32) -> (Point3<f32>, Vector3<f32>) {
        // Convert screen coordinates to NDC
        let ndc_x = (screen_x / self.viewport_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / self.viewport_height) * 2.0;
        
        // Inverse projection
        let inv_proj = self.projection_matrix.try_inverse().unwrap();
        let view_pos = inv_proj * nalgebra::Vector4::new(ndc_x, ndc_y, -1.0, 1.0);
        let view_dir = Vector3::new(view_pos.x / view_pos.w, view_pos.y / view_pos.w, -1.0);
        
        // Transform to world space
        let inv_view = self.view_matrix.try_inverse().unwrap();
        let world_origin_4d = inv_view * Point3::new(0.0, 0.0, 0.0).to_homogeneous();
        let world_origin = Point3::from_homogeneous(world_origin_4d).unwrap();
        let view_dir_4d = Vector4::new(view_dir.x, view_dir.y, view_dir.z, 0.0);
        let world_dir_4d = inv_view * view_dir_4d;
        let world_dir = Vector3::new(world_dir_4d.x, world_dir_4d.y, world_dir_4d.z).normalize();
        
        (world_origin, world_dir)
    }

    // Getters
    pub fn get_position(&self) -> Point3<f32> {
        self.position
    }

    pub fn get_target(&self) -> Point3<f32> {
        self.target
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
    }

    pub fn get_view_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix * self.view_matrix
    }

    pub fn get_camera_mode(&self) -> &CameraMode {
        &self.current_mode
    }

    pub fn get_forward_vector(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }

    pub fn get_right_vector(&self) -> Vector3<f32> {
        self.get_forward_vector().cross(&self.up).normalize()
    }

    pub fn get_up_vector(&self) -> Vector3<f32> {
        self.up
    }

    // Build mode specific functions
    pub fn set_build_focus_point(&mut self, point: Point3<f32>) {
        self.build_focus_point = point;
    }

    pub fn get_build_focus_point(&self) -> Point3<f32> {
        self.build_focus_point
    }

    pub fn zoom_to_area(&mut self, min_bound: Point3<f32>, max_bound: Point3<f32>) {
        let center = Point3::new(
            (min_bound.x + max_bound.x) * 0.5,
            (min_bound.y + max_bound.y) * 0.5,
            (min_bound.z + max_bound.z) * 0.5,
        );
        
        let size = (max_bound - min_bound).magnitude();
        let distance = size / (2.0 * (self.fov * 0.5).tan());
        
        self.build_focus_point = center;
        self.build_orbit_distance = distance * 1.2; // Add some padding
    }
}