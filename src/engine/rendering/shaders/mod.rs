/*!
 * Robin Engine Shader Management
 * 
 * Shader compilation, loading, and management for the rendering pipeline.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    graphics::GraphicsContext,
};
use std::collections::HashMap;

/// Shader program manager
#[derive(Debug)]
pub struct ShaderManager {
    programs: HashMap<String, ShaderProgram>,
    shader_cache: HashMap<String, CompiledShader>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
            shader_cache: HashMap::new(),
        }
    }

    /// Load and compile a shader program
    pub fn load_program(&mut self, name: &str, vertex_src: &str, fragment_src: &str) -> RobinResult<u32> {
        let vertex_shader = self.compile_shader(vertex_src, gl::VERTEX_SHADER)?;
        let fragment_shader = self.compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;
        
        let program = self.link_program(vertex_shader, fragment_shader)?;
        
        let shader_program = ShaderProgram {
            handle: program,
            vertex_shader,
            fragment_shader,
            uniforms: HashMap::new(),
        };
        
        self.programs.insert(name.to_string(), shader_program);
        Ok(program)
    }

    /// Get a shader program by name
    pub fn get_program(&self, name: &str) -> Option<u32> {
        self.programs.get(name).map(|p| p.handle)
    }

    fn compile_shader(&mut self, source: &str, shader_type: u32) -> RobinResult<u32> {
        let shader = unsafe { gl::CreateShader(shader_type) };
        
        unsafe {
            let c_str = std::ffi::CString::new(source)
                .map_err(|_| RobinError::GraphicsError("Invalid shader source".to_string()))?;
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);
            
            // Check compilation status
            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                
                let mut log = Vec::with_capacity(len as usize);
                gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _);
                log.set_len(len as usize);
                
                let error_msg = String::from_utf8_lossy(&log);
                return Err(RobinError::GraphicsError(format!("Shader compilation failed: {}", error_msg)));
            }
        }
        
        Ok(shader)
    }

    fn link_program(&self, vertex_shader: u32, fragment_shader: u32) -> RobinResult<u32> {
        let program = unsafe { gl::CreateProgram() };
        
        unsafe {
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
            
            // Check linking status
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                
                let mut log = Vec::with_capacity(len as usize);
                gl::GetProgramInfoLog(program, len, std::ptr::null_mut(), log.as_mut_ptr() as *mut _);
                log.set_len(len as usize);
                
                let error_msg = String::from_utf8_lossy(&log);
                return Err(RobinError::GraphicsError(format!("Program linking failed: {}", error_msg)));
            }
        }
        
        Ok(program)
    }
}

/// Individual shader program
#[derive(Debug)]
pub struct ShaderProgram {
    pub handle: u32,
    pub vertex_shader: u32,
    pub fragment_shader: u32,
    pub uniforms: HashMap<String, i32>,
}

impl ShaderProgram {
    /// Get uniform location, caching the result
    pub fn get_uniform_location(&mut self, name: &str) -> i32 {
        if let Some(&location) = self.uniforms.get(name) {
            return location;
        }
        
        let c_name = std::ffi::CString::new(name).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.handle, c_name.as_ptr()) };
        self.uniforms.insert(name.to_string(), location);
        location
    }

    /// Bind the shader program
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.handle);
        }
    }

    /// Unbind shader programs
    pub fn unbind() {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

/// Compiled shader cache entry
#[derive(Debug)]
pub struct CompiledShader {
    pub handle: u32,
    pub shader_type: u32,
    pub source_hash: u64,
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.vertex_shader);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteProgram(self.handle);
        }
    }
}