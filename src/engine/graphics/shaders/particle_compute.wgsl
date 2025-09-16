struct Particle {
    position: vec4<f32>,        // xyz + life
    velocity: vec4<f32>,        // xyz + mass
    color: vec4<f32>,           // rgba
    size_rotation: vec4<f32>,   // size, rotation, angular_vel, padding
}

struct ParticleSystemUniforms {
    emitter_position: vec4<f32>,
    emitter_direction: vec4<f32>,
    gravity: vec4<f32>,
    delta_time: f32,
    spawn_rate: f32,
    max_particles: u32,
    current_time: f32,
    color_start: vec4<f32>,
    color_end: vec4<f32>,
    size_curve: vec4<f32>,      // x=start, y=end, z=curve_power, w=padding
    emission_params: vec4<f32>, // shape-specific parameters
}

struct IndirectDrawCommand {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> uniforms: ParticleSystemUniforms;

@group(0) @binding(2)
var<storage, read_write> indirect_draw: IndirectDrawCommand;

@group(0) @binding(3)
var<storage, read_write> particle_count: atomic<u32>;

// Pseudo-random number generation
var<private> rng_seed: u32;

fn rng_init(id: u32) {
    rng_seed = id * 747796405u + 2891336453u;
}

fn rng() -> f32 {
    rng_seed = rng_seed * 747796405u + 2891336453u;
    let result = ((rng_seed >> ((rng_seed >> 28u) + 4u)) ^ rng_seed) * 277803737u;
    return f32((result >> 22u) ^ result) / 4294967295.0;
}

fn rng_range(min_val: f32, max_val: f32) -> f32 {
    return min_val + rng() * (max_val - min_val);
}

fn rng_vec3_sphere(radius: f32) -> vec3<f32> {
    let theta = rng() * 6.28318530718; // 2 * PI
    let phi = acos(1.0 - 2.0 * rng());
    let r = pow(rng(), 1.0 / 3.0) * radius;
    
    return vec3<f32>(
        r * sin(phi) * cos(theta),
        r * sin(phi) * sin(theta),
        r * cos(phi)
    );
}

fn rng_vec3_box(size: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        rng_range(-size.x * 0.5, size.x * 0.5),
        rng_range(-size.y * 0.5, size.y * 0.5),
        rng_range(-size.z * 0.5, size.z * 0.5)
    );
}

fn rng_vec3_cone(radius: f32, angle_degrees: f32) -> vec3<f32> {
    let angle_rad = angle_degrees * 0.0174532925199; // degrees to radians
    let theta = rng() * 6.28318530718; // 2 * PI
    let phi = rng() * angle_rad;
    let r = sqrt(rng()) * radius;
    
    return vec3<f32>(
        r * sin(phi) * cos(theta),
        cos(phi),
        r * sin(phi) * sin(theta)
    );
}

fn rng_vec3_circle(radius: f32) -> vec3<f32> {
    let theta = rng() * 6.28318530718; // 2 * PI
    let r = sqrt(rng()) * radius;
    
    return vec3<f32>(
        r * cos(theta),
        0.0,
        r * sin(theta)
    );
}

fn get_emission_position() -> vec3<f32> {
    let shape_type = u32(uniforms.emission_params.w);
    
    switch shape_type {
        case 0u: { // Point
            return vec3<f32>(0.0, 0.0, 0.0);
        }
        case 1u: { // Sphere
            let radius = uniforms.emission_params.x;
            return rng_vec3_sphere(radius);
        }
        case 2u: { // Box
            let size = uniforms.emission_params.xyz;
            return rng_vec3_box(size);
        }
        case 3u: { // Cone
            let radius = uniforms.emission_params.x;
            let angle = uniforms.emission_params.y;
            return rng_vec3_cone(radius, angle);
        }
        case 4u: { // Circle
            let radius = uniforms.emission_params.x;
            return rng_vec3_circle(radius);
        }
        default: {
            return vec3<f32>(0.0, 0.0, 0.0);
        }
    }
}

fn get_emission_velocity() -> vec3<f32> {
    let base_direction = normalize(uniforms.emitter_direction.xyz);
    let shape_type = u32(uniforms.emission_params.w);
    
    var direction = base_direction;
    
    // Add some randomness to the direction
    let random_offset = vec3<f32>(
        rng_range(-1.0, 1.0),
        rng_range(-1.0, 1.0),
        rng_range(-1.0, 1.0)
    ) * 0.3;
    
    direction = normalize(direction + random_offset);
    
    // Apply initial speed with variance
    let speed = rng_range(5.0, 15.0); // This should come from uniforms
    
    return direction * speed;
}

fn spawn_particle(index: u32) {
    let local_position = get_emission_position();
    let world_position = uniforms.emitter_position.xyz + local_position;
    let velocity = get_emission_velocity();
    
    let lifetime = rng_range(2.0, 5.0); // This should come from uniforms
    let size = rng_range(0.5, 2.0);     // This should come from uniforms
    
    particles[index].position = vec4<f32>(world_position, lifetime);
    particles[index].velocity = vec4<f32>(velocity, 1.0); // mass = 1.0
    particles[index].color = uniforms.color_start;
    particles[index].size_rotation = vec4<f32>(size, 0.0, rng_range(-3.14, 3.14), 0.0);
}

fn update_particle(index: u32) -> bool {
    var particle = particles[index];
    
    // Check if particle is alive
    if particle.position.w <= 0.0 {
        return false;
    }
    
    // Update position
    particle.position.x += particle.velocity.x * uniforms.delta_time;
    particle.position.y += particle.velocity.y * uniforms.delta_time;
    particle.position.z += particle.velocity.z * uniforms.delta_time;
    
    // Apply gravity
    particle.velocity.x += uniforms.gravity.x * uniforms.delta_time;
    particle.velocity.y += uniforms.gravity.y * uniforms.delta_time;
    particle.velocity.z += uniforms.gravity.z * uniforms.delta_time;
    
    // Update lifetime
    particle.position.w -= uniforms.delta_time;
    
    // Update color based on lifetime
    let life_ratio = particle.position.w / 3.0; // Max lifetime, should be from uniforms
    particle.color = mix(uniforms.color_end, uniforms.color_start, life_ratio);
    
    // Update size based on lifetime
    let size_ratio = smoothstep(0.0, 1.0, life_ratio);
    particle.size_rotation.x = mix(uniforms.size_curve.y, uniforms.size_curve.x, size_ratio);
    
    // Update rotation
    particle.size_rotation.y += particle.size_rotation.z * uniforms.delta_time;
    
    // Store updated particle
    particles[index] = particle;
    
    return particle.position.w > 0.0;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    
    if index >= uniforms.max_particles {
        return;
    }
    
    // Initialize RNG for this thread
    rng_init(index + u32(uniforms.current_time * 1000.0));
    
    // Update existing particle
    let is_alive = update_particle(index);
    
    // If particle died, try to spawn a new one
    if !is_alive {
        // Simple spawning logic - spawn based on spawn rate
        let spawn_probability = uniforms.spawn_rate * uniforms.delta_time / f32(uniforms.max_particles);
        if rng() < spawn_probability {
            spawn_particle(index);
            atomicAdd(&particle_count, 1u);
        }
    } else {
        atomicAdd(&particle_count, 1u);
    }
    
    // Update indirect draw command (only first thread)
    if index == 0u {
        let alive_count = atomicLoad(&particle_count);
        indirect_draw.vertex_count = 4u;  // Triangle strip quad
        indirect_draw.instance_count = alive_count;
        indirect_draw.first_vertex = 0u;
        indirect_draw.first_instance = 0u;
        
        // Reset counter for next frame
        atomicStore(&particle_count, 0u);
    }
}