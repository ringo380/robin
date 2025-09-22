use cgmath::{Vector3, Matrix4, Point3, Quaternion, Rotation3, InnerSpace, Zero, One};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Door {
        open_angle: f32,
        open_speed: f32,
        auto_close_delay: Option<f32>,
        requires_key: Option<String>,
    },
    Platform {
        movement_type: PlatformMovement,
        speed: f32,
        wait_time: f32,
        auto_activate: bool,
    },
    Trigger {
        trigger_type: TriggerType,
        shape: TriggerShape,
        one_shot: bool,
        target_elements: Vec<u32>,
    },
    Switch {
        switch_type: SwitchType,
        toggle_mode: bool,
        activation_sound: Option<String>,
    },
    Collectible {
        collectible_type: CollectibleType,
        value: i32,
        respawn_time: Option<f32>,
        pickup_sound: Option<String>,
    },
    Hazard {
        hazard_type: HazardType,
        damage: i32,
        effect_radius: f32,
        warning_time: f32,
    },
    Container {
        capacity: u32,
        locked: bool,
        key_required: Option<String>,
        contents: Vec<ItemStack>,
    },
    Teleporter {
        destination: Vector3<f32>,
        destination_id: Option<u32>,
        activation_delay: f32,
        two_way: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformMovement {
    Linear { waypoints: Vec<Vector3<f32>> },
    Circular { center: Vector3<f32>, radius: f32 },
    Elevator { min_height: f32, max_height: f32 },
    Pendulum { pivot: Vector3<f32>, max_angle: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    PlayerEnter,
    PlayerExit,
    PlayerStay,
    ItemDrop,
    ProjectileHit,
    TimeDelay(f32),
    HealthThreshold(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerShape {
    Box { size: Vector3<f32> },
    Sphere { radius: f32 },
    Cylinder { radius: f32, height: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchType {
    Button,
    Lever,
    PressurePlate,
    ProximitySensor,
    Timer(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectibleType {
    Coin,
    Gem,
    Key(String),
    Health,
    Ammo,
    PowerUp(String),
    QuestItem(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HazardType {
    Spikes,
    Fire,
    Electricity,
    Poison,
    Laser,
    MovingBlade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub id: u32,
    pub element_type: ElementType,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub enabled: bool,
    pub state: ElementState,
    pub properties: HashMap<String, ElementProperty>,
    pub connections: Vec<ElementConnection>,
    pub animation_state: AnimationState,
    pub visual_mesh: String,
    pub collision_shape: CollisionShape,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementState {
    Inactive,
    Active,
    Triggered,
    Disabled,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementProperty {
    Float(f32),
    Int(i32),
    String(String),
    Bool(bool),
    Vector3(Vector3<f32>),
    Color([f32; 4]),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementConnection {
    pub target_id: u32,
    pub connection_type: ConnectionType,
    pub delay: f32,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Activate,
    Deactivate,
    Toggle,
    Trigger,
    Signal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationState {
    pub current_frame: f32,
    pub playing: bool,
    pub loop_animation: bool,
    pub speed_multiplier: f32,
    pub keyframes: Vec<AnimationKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationKeyframe {
    pub time: f32,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub properties: HashMap<String, ElementProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionShape {
    Box { size: Vector3<f32> },
    Sphere { radius: f32 },
    Cylinder { radius: f32, height: f32 },
    Mesh { vertices: Vec<Vector3<f32>>, indices: Vec<u32> },
    Compound { shapes: Vec<(Vector3<f32>, CollisionShape)> },
}

pub struct InteractiveElementsSystem {
    elements: HashMap<u32, InteractiveElement>,
    next_id: u32,
    element_templates: HashMap<String, ElementTemplate>,
    active_animations: Vec<u32>,
    trigger_queue: Vec<TriggerEvent>,
    sound_system: Option<Box<dyn SoundSystem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
    pub element_type: ElementType,
    pub default_scale: Vector3<f32>,
    pub visual_mesh: String,
    pub collision_shape: CollisionShape,
    pub properties: HashMap<String, ElementProperty>,
    pub preview_icon: String,
}

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub element_id: u32,
    pub trigger_type: TriggerType,
    pub data: HashMap<String, ElementProperty>,
    pub timestamp: f32,
}

pub trait SoundSystem {
    fn play_sound(&mut self, sound_name: &str, position: Vector3<f32>, volume: f32);
    fn stop_sound(&mut self, sound_name: &str);
    fn set_listener_position(&mut self, position: Vector3<f32>, forward: Vector3<f32>);
}

impl InteractiveElementsSystem {
    pub fn new() -> Self {
        let mut system = Self {
            elements: HashMap::new(),
            next_id: 1,
            element_templates: HashMap::new(),
            active_animations: Vec::new(),
            trigger_queue: Vec::new(),
            sound_system: None,
        };

        system.initialize_default_templates();
        system
    }

    pub fn add_element(&mut self, template_name: &str, position: Vector3<f32>) -> Result<u32, String> {
        let template = self.element_templates.get(template_name)
            .ok_or_else(|| format!("Template '{}' not found", template_name))?;

        let id = self.next_id;
        self.next_id += 1;

        let element = InteractiveElement {
            id,
            element_type: template.element_type.clone(),
            position,
            rotation: Quaternion::one(),
            scale: template.default_scale,
            enabled: true,
            state: ElementState::Inactive,
            properties: template.properties.clone(),
            connections: Vec::new(),
            animation_state: AnimationState {
                current_frame: 0.0,
                playing: false,
                loop_animation: false,
                speed_multiplier: 1.0,
                keyframes: Vec::new(),
            },
            visual_mesh: template.visual_mesh.clone(),
            collision_shape: template.collision_shape.clone(),
            tags: Vec::new(),
        };

        self.elements.insert(id, element);
        Ok(id)
    }

    pub fn remove_element(&mut self, id: u32) -> Result<(), String> {
        self.elements.remove(&id)
            .ok_or_else(|| format!("Element {} not found", id))?;

        // Remove from active animations
        self.active_animations.retain(|&anim_id| anim_id != id);

        // Remove connections to this element
        for element in self.elements.values_mut() {
            element.connections.retain(|conn| conn.target_id != id);
        }

        Ok(())
    }

    pub fn connect_elements(&mut self, source_id: u32, target_id: u32, connection_type: ConnectionType) -> Result<(), String> {
        if !self.elements.contains_key(&source_id) {
            return Err(format!("Source element {} not found", source_id));
        }
        if !self.elements.contains_key(&target_id) {
            return Err(format!("Target element {} not found", target_id));
        }

        let connection = ElementConnection {
            target_id,
            connection_type,
            delay: 0.0,
            condition: None,
        };

        if let Some(source) = self.elements.get_mut(&source_id) {
            source.connections.push(connection);
        }

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32, player_position: Vector3<f32>) {
        self.update_animations(delta_time);
        self.update_triggers(delta_time, player_position);
        self.process_trigger_queue(delta_time);
        self.update_element_logic(delta_time);
    }

    fn update_animations(&mut self, delta_time: f32) {
        for &element_id in &self.active_animations.clone() {
            // First, extract the animation state and calculate values outside the mutable borrow
            let (anim_state, duration, transform) = {
                if let Some(element) = self.elements.get(&element_id) {
                    if element.animation_state.playing {
                        let mut new_anim_state = element.animation_state.clone();
                        new_anim_state.current_frame += delta_time * new_anim_state.speed_multiplier;

                        let duration = self.get_animation_duration(&new_anim_state);
                        let transform = self.interpolate_animation(&new_anim_state);

                        (Some(new_anim_state), duration, transform)
                    } else {
                        (None, 0.0, None)
                    }
                } else {
                    (None, 0.0, None)
                }
            };

            // Now apply the changes with a mutable borrow
            if let (Some(anim_state), Some(element)) = (anim_state, self.elements.get_mut(&element_id)) {
                // Update animation state
                element.animation_state = anim_state;

                // Apply transform if available
                if let Some(transform) = transform {
                    element.position = transform.0;
                    element.rotation = transform.1;
                    element.scale = transform.2;
                }

                // Check if animation finished
                if !element.animation_state.loop_animation &&
                   element.animation_state.current_frame >= duration {
                    element.animation_state.playing = false;
                    self.active_animations.retain(|&id| id != element_id);
                }
            }
        }
    }

    fn update_triggers(&mut self, delta_time: f32, player_position: Vector3<f32>) {
        let mut new_triggers = Vec::new();

        // First pass: collect trigger data to avoid borrow conflicts
        let trigger_data: Vec<(u32, bool, ElementState)> = self.elements
            .values()
            .filter_map(|element| {
                if let ElementType::Trigger { trigger_type, shape, one_shot, .. } = &element.element_type {
                    if element.enabled && (!*one_shot || element.state != ElementState::Triggered) {
                        match trigger_type {
                            TriggerType::PlayerEnter | TriggerType::PlayerStay => {
                                let in_trigger = self.point_in_trigger_shape(player_position, element.position, shape);
                                Some((element.id, in_trigger, element.state.clone()))
                            }
                            _ => None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Second pass: update elements based on collected data
        for (element_id, in_trigger, current_state) in trigger_data {
            if let Some(element) = self.elements.get_mut(&element_id) {
                if let ElementType::Trigger { trigger_type, .. } = &element.element_type {
                    match trigger_type {
                        TriggerType::PlayerEnter | TriggerType::PlayerStay => {
                            if in_trigger {
                                if current_state != ElementState::Active {
                                    new_triggers.push(TriggerEvent {
                                        element_id,
                                        trigger_type: trigger_type.clone(),
                                        data: HashMap::new(),
                                        timestamp: 0.0,
                                    });
                                    element.state = ElementState::Active;
                                }
                            } else if current_state == ElementState::Active {
                                if matches!(trigger_type, TriggerType::PlayerExit) {
                                    new_triggers.push(TriggerEvent {
                                        element_id,
                                        trigger_type: trigger_type.clone(),
                                        data: HashMap::new(),
                                        timestamp: 0.0,
                                    });
                                }
                                element.state = ElementState::Inactive;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Handle time-based triggers separately
        for element in self.elements.values_mut() {
            if let ElementType::Trigger { trigger_type, one_shot, .. } = &element.element_type {
                if element.enabled && (!*one_shot || element.state != ElementState::Triggered) {
                    if let TriggerType::TimeDelay(delay) = trigger_type {
                        if element.state == ElementState::Active {
                            if let Some(ElementProperty::Float(timer)) = element.properties.get_mut("timer") {
                                *timer += delta_time;
                                if *timer >= *delay {
                                    new_triggers.push(TriggerEvent {
                                        element_id: element.id,
                                        trigger_type: trigger_type.clone(),
                                        data: HashMap::new(),
                                        timestamp: 0.0,
                                    });
                                    element.state = ElementState::Triggered;
                                    *timer = 0.0;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.trigger_queue.extend(new_triggers);
    }

    fn process_trigger_queue(&mut self, _delta_time: f32) {
        // Collect all trigger events and their connections first
        let trigger_events: Vec<TriggerEvent> = self.trigger_queue.drain(..).collect();

        for trigger_event in trigger_events {
            if let Some(element) = self.elements.get(&trigger_event.element_id) {
                // Clone connections to avoid borrow conflicts
                let connections = element.connections.clone();
                for connection in &connections {
                    self.execute_connection(connection, &trigger_event);
                }
            }
        }
    }

    fn execute_connection(&mut self, connection: &ElementConnection, _trigger_event: &TriggerEvent) {
        // Store the target_id to activate after updating the element state
        let mut activate_element_id = None;

        if let Some(target) = self.elements.get_mut(&connection.target_id) {
            match &connection.connection_type {
                ConnectionType::Activate => {
                    target.state = ElementState::Active;
                    activate_element_id = Some(target.id);
                }
                ConnectionType::Deactivate => {
                    target.state = ElementState::Inactive;
                }
                ConnectionType::Toggle => {
                    target.state = match target.state {
                        ElementState::Active => ElementState::Inactive,
                        ElementState::Inactive => ElementState::Active,
                        _ => target.state.clone(),
                    };
                }
                ConnectionType::Trigger => {
                    target.state = ElementState::Triggered;
                }
                ConnectionType::Signal(signal) => {
                    target.properties.insert(
                        "signal".to_string(),
                        ElementProperty::String(signal.clone())
                    );
                }
            }
        }

        // Activate element after releasing the mutable borrow
        if let Some(element_id) = activate_element_id {
            self.activate_element(element_id);
        }
    }

    fn activate_element(&mut self, element_id: u32) {
        if let Some(element) = self.elements.get_mut(&element_id) {
            match &element.element_type {
                ElementType::Door { open_speed, .. } => {
                    if !element.animation_state.playing {
                        element.animation_state.playing = true;
                        element.animation_state.speed_multiplier = *open_speed;
                        if !self.active_animations.contains(&element_id) {
                            self.active_animations.push(element_id);
                        }
                    }
                }
                ElementType::Platform { speed, .. } => {
                    element.animation_state.playing = true;
                    element.animation_state.speed_multiplier = *speed;
                    element.animation_state.loop_animation = true;
                    if !self.active_animations.contains(&element_id) {
                        self.active_animations.push(element_id);
                    }
                }
                ElementType::Teleporter { destination, activation_delay, .. } => {
                    // Store teleport destination for player
                    element.properties.insert(
                        "teleport_destination".to_string(),
                        ElementProperty::Vector3(*destination)
                    );
                    element.properties.insert(
                        "teleport_delay".to_string(),
                        ElementProperty::Float(*activation_delay)
                    );
                }
                _ => {}
            }
        }
    }

    fn update_element_logic(&mut self, delta_time: f32) {
        for element in self.elements.values_mut() {
            match &element.element_type {
                ElementType::Door { auto_close_delay, .. } => {
                    if let Some(delay) = auto_close_delay {
                        if element.state == ElementState::Active {
                            if let Some(ElementProperty::Float(timer)) = element.properties.get_mut("auto_close_timer") {
                                *timer += delta_time;
                                if *timer >= *delay {
                                    element.state = ElementState::Inactive;
                                    element.animation_state.playing = true;
                                    element.animation_state.speed_multiplier = -1.0; // Reverse animation
                                    *timer = 0.0;
                                }
                            } else {
                                element.properties.insert("auto_close_timer".to_string(), ElementProperty::Float(0.0));
                            }
                        }
                    }
                }
                ElementType::Collectible { respawn_time, .. } => {
                    if element.state == ElementState::Triggered {
                        if let Some(respawn) = respawn_time {
                            if let Some(ElementProperty::Float(timer)) = element.properties.get_mut("respawn_timer") {
                                *timer += delta_time;
                                if *timer >= *respawn {
                                    element.state = ElementState::Inactive;
                                    element.enabled = true;
                                    *timer = 0.0;
                                }
                            } else {
                                element.properties.insert("respawn_timer".to_string(), ElementProperty::Float(0.0));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn point_in_trigger_shape(&self, point: Vector3<f32>, trigger_pos: Vector3<f32>, shape: &TriggerShape) -> bool {
        let relative_pos = point - trigger_pos;

        match shape {
            TriggerShape::Box { size } => {
                relative_pos.x.abs() <= size.x / 2.0 &&
                relative_pos.y.abs() <= size.y / 2.0 &&
                relative_pos.z.abs() <= size.z / 2.0
            }
            TriggerShape::Sphere { radius } => {
                relative_pos.magnitude() <= *radius
            }
            TriggerShape::Cylinder { radius, height } => {
                let horizontal_dist = (relative_pos.x * relative_pos.x + relative_pos.z * relative_pos.z).sqrt();
                horizontal_dist <= *radius && relative_pos.y.abs() <= height / 2.0
            }
        }
    }

    fn interpolate_animation(&self, anim_state: &AnimationState) -> Option<(Vector3<f32>, Quaternion<f32>, Vector3<f32>)> {
        if anim_state.keyframes.len() < 2 {
            return None;
        }

        let duration = self.get_animation_duration(anim_state);
        let normalized_time = (anim_state.current_frame % duration) / duration;

        // Find surrounding keyframes
        for i in 0..anim_state.keyframes.len() - 1 {
            let current_kf = &anim_state.keyframes[i];
            let next_kf = &anim_state.keyframes[i + 1];

            if normalized_time >= current_kf.time && normalized_time <= next_kf.time {
                let local_t = (normalized_time - current_kf.time) / (next_kf.time - current_kf.time);

                let position = current_kf.position + (next_kf.position - current_kf.position) * local_t;
                let rotation = current_kf.rotation.slerp(next_kf.rotation, local_t);
                let scale = current_kf.scale + (next_kf.scale - current_kf.scale) * local_t;

                return Some((position, rotation, scale));
            }
        }

        None
    }

    fn get_animation_duration(&self, anim_state: &AnimationState) -> f32 {
        anim_state.keyframes.last().map(|kf| kf.time).unwrap_or(1.0)
    }

    fn initialize_default_templates(&mut self) {
        // Door template
        self.element_templates.insert("basic_door".to_string(), ElementTemplate {
            name: "Basic Door".to_string(),
            description: "A simple hinged door that opens when triggered".to_string(),
            category: "Interactive".to_string(),
            element_type: ElementType::Door {
                open_angle: 90.0,
                open_speed: 2.0,
                auto_close_delay: Some(3.0),
                requires_key: None,
            },
            default_scale: Vector3::new(1.0, 2.0, 0.1),
            visual_mesh: "door_basic.obj".to_string(),
            collision_shape: CollisionShape::Box { size: Vector3::new(1.0, 2.0, 0.1) },
            properties: HashMap::new(),
            preview_icon: "door_icon.png".to_string(),
        });

        // Moving platform template
        self.element_templates.insert("moving_platform".to_string(), ElementTemplate {
            name: "Moving Platform".to_string(),
            description: "A platform that moves between waypoints".to_string(),
            category: "Interactive".to_string(),
            element_type: ElementType::Platform {
                movement_type: PlatformMovement::Linear {
                    waypoints: vec![Vector3::zero(), Vector3::new(5.0, 0.0, 0.0)]
                },
                speed: 1.0,
                wait_time: 1.0,
                auto_activate: true,
            },
            default_scale: Vector3::new(2.0, 0.2, 2.0),
            visual_mesh: "platform.obj".to_string(),
            collision_shape: CollisionShape::Box { size: Vector3::new(2.0, 0.2, 2.0) },
            properties: HashMap::new(),
            preview_icon: "platform_icon.png".to_string(),
        });

        // Pressure plate template
        self.element_templates.insert("pressure_plate".to_string(), ElementTemplate {
            name: "Pressure Plate".to_string(),
            description: "Activates when stepped on".to_string(),
            category: "Triggers".to_string(),
            element_type: ElementType::Switch {
                switch_type: SwitchType::PressurePlate,
                toggle_mode: false,
                activation_sound: Some("click.wav".to_string()),
            },
            default_scale: Vector3::new(1.0, 0.1, 1.0),
            visual_mesh: "pressure_plate.obj".to_string(),
            collision_shape: CollisionShape::Box { size: Vector3::new(1.0, 0.1, 1.0) },
            properties: HashMap::new(),
            preview_icon: "pressure_plate_icon.png".to_string(),
        });

        // Coin collectible template
        self.element_templates.insert("coin".to_string(), ElementTemplate {
            name: "Coin".to_string(),
            description: "A collectible coin worth points".to_string(),
            category: "Collectibles".to_string(),
            element_type: ElementType::Collectible {
                collectible_type: CollectibleType::Coin,
                value: 10,
                respawn_time: None,
                pickup_sound: Some("coin_pickup.wav".to_string()),
            },
            default_scale: Vector3::new(0.3, 0.3, 0.1),
            visual_mesh: "coin.obj".to_string(),
            collision_shape: CollisionShape::Cylinder { radius: 0.15, height: 0.1 },
            properties: HashMap::new(),
            preview_icon: "coin_icon.png".to_string(),
        });

        // Spike hazard template
        self.element_templates.insert("spikes".to_string(), ElementTemplate {
            name: "Spikes".to_string(),
            description: "Sharp spikes that damage the player".to_string(),
            category: "Hazards".to_string(),
            element_type: ElementType::Hazard {
                hazard_type: HazardType::Spikes,
                damage: 25,
                effect_radius: 0.5,
                warning_time: 0.5,
            },
            default_scale: Vector3::new(1.0, 0.5, 1.0),
            visual_mesh: "spikes.obj".to_string(),
            collision_shape: CollisionShape::Box { size: Vector3::new(1.0, 0.5, 1.0) },
            properties: HashMap::new(),
            preview_icon: "spikes_icon.png".to_string(),
        });

        // Teleporter template
        self.element_templates.insert("teleporter".to_string(), ElementTemplate {
            name: "Teleporter".to_string(),
            description: "Teleports player to another location".to_string(),
            category: "Interactive".to_string(),
            element_type: ElementType::Teleporter {
                destination: Vector3::new(10.0, 0.0, 10.0),
                destination_id: None,
                activation_delay: 1.0,
                two_way: false,
            },
            default_scale: Vector3::new(1.5, 0.1, 1.5),
            visual_mesh: "teleporter.obj".to_string(),
            collision_shape: CollisionShape::Cylinder { radius: 0.75, height: 0.1 },
            properties: HashMap::new(),
            preview_icon: "teleporter_icon.png".to_string(),
        });
    }

    pub fn get_templates(&self) -> &HashMap<String, ElementTemplate> {
        &self.element_templates
    }

    pub fn get_elements(&self) -> &HashMap<u32, InteractiveElement> {
        &self.elements
    }

    pub fn get_element_mut(&mut self, id: u32) -> Option<&mut InteractiveElement> {
        self.elements.get_mut(&id)
    }

    pub fn set_sound_system(&mut self, sound_system: Box<dyn SoundSystem>) {
        self.sound_system = Some(sound_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let mut system = InteractiveElementsSystem::new();
        let id = system.add_element("basic_door", Vector3::new(0.0, 0.0, 0.0)).unwrap();
        assert!(system.elements.contains_key(&id));
    }

    #[test]
    fn test_element_connection() {
        let mut system = InteractiveElementsSystem::new();
        let door_id = system.add_element("basic_door", Vector3::new(0.0, 0.0, 0.0)).unwrap();
        let switch_id = system.add_element("pressure_plate", Vector3::new(1.0, 0.0, 0.0)).unwrap();

        system.connect_elements(switch_id, door_id, ConnectionType::Activate).unwrap();

        let switch = system.elements.get(&switch_id).unwrap();
        assert_eq!(switch.connections.len(), 1);
        assert_eq!(switch.connections[0].target_id, door_id);
    }

    #[test]
    fn test_trigger_detection() {
        let system = InteractiveElementsSystem::new();
        let shape = TriggerShape::Box { size: Vector3::new(2.0, 2.0, 2.0) };
        let trigger_pos = Vector3::new(0.0, 0.0, 0.0);

        assert!(system.point_in_trigger_shape(Vector3::new(0.5, 0.5, 0.5), trigger_pos, &shape));
        assert!(!system.point_in_trigger_shape(Vector3::new(2.0, 2.0, 2.0), trigger_pos, &shape));
    }
}