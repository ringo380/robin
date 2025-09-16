pub struct Shader {
    // Will hold shader data
}

impl Shader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn from_source(vertex: &str, fragment: &str) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Creating shader from source");
        Ok(Self::new())
    }
}