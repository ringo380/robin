use std::collections::HashMap;
use rand::Rng;
use crate::engine::story::{Quest, StoryEvent, Character};

#[derive(Debug, Clone)]
pub struct StoryGenerator {
    story_templates: HashMap<String, StoryTemplate>,
    character_archetypes: HashMap<String, CharacterArchetype>,
    plot_devices: Vec<PlotDevice>,
    story_seeds: Vec<StorySeed>,
    generation_rules: Vec<GenerationRule>,
    narrative_patterns: HashMap<String, NarrativePattern>,
}

#[derive(Debug, Clone)]
pub struct StoryTemplate {
    pub id: String,
    pub name: String,
    pub genre: Genre,
    pub structure: StoryStructure,
    pub required_elements: Vec<StoryElement>,
    pub optional_elements: Vec<StoryElement>,
    pub character_roles: Vec<CharacterRole>,
    pub estimated_duration: u32,
    pub complexity_level: f32,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Genre {
    Adventure,
    Mystery,
    Romance,
    Horror,
    Comedy,
    Drama,
    Fantasy,
    SciFi,
    Thriller,
    Historical,
}

#[derive(Debug, Clone)]
pub struct StoryStructure {
    pub acts: Vec<Act>,
    pub pacing_curve: Vec<f32>,
    pub tension_points: Vec<TensionPoint>,
}

#[derive(Debug, Clone)]
pub struct Act {
    pub name: String,
    pub relative_length: f32,
    pub key_events: Vec<String>,
    pub character_focuses: Vec<String>,
    pub mood: ActMood,
}

#[derive(Debug, Clone)]
pub enum ActMood {
    Setup,
    Building,
    Climactic,
    Resolution,
}

#[derive(Debug, Clone)]
pub struct TensionPoint {
    pub position: f32,
    pub intensity: f32,
    pub event_type: TensionEventType,
}

#[derive(Debug, Clone)]
pub enum TensionEventType {
    Conflict,
    Revelation,
    Betrayal,
    Loss,
    Discovery,
    Confrontation,
}

#[derive(Debug, Clone)]
pub struct StoryElement {
    pub element_type: ElementType,
    pub description: String,
    pub requirements: Vec<String>,
    pub provides: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ElementType {
    MacGuffin,
    Conflict,
    Mystery,
    Romance,
    Betrayal,
    Sacrifice,
    Discovery,
    Transformation,
}

#[derive(Debug, Clone)]
pub struct CharacterRole {
    pub role_name: String,
    pub importance: RoleImportance,
    pub arc_requirements: Vec<String>,
    pub relationship_requirements: Vec<RelationshipRequirement>,
}

#[derive(Debug, Clone)]
pub enum RoleImportance {
    Protagonist,
    MainSupporting,
    Supporting,
    Minor,
}

#[derive(Debug, Clone)]
pub struct RelationshipRequirement {
    pub other_role: String,
    pub relationship_type: String,
    pub development: RelationshipDevelopment,
}

#[derive(Debug, Clone)]
pub enum RelationshipDevelopment {
    Static,
    Improving,
    Deteriorating,
    Complex,
}

#[derive(Debug, Clone)]
pub struct CharacterArchetype {
    pub name: String,
    pub core_traits: Vec<String>,
    pub motivations: Vec<String>,
    pub typical_roles: Vec<String>,
    pub story_functions: Vec<StoryFunction>,
}

#[derive(Debug, Clone)]
pub enum StoryFunction {
    Catalyst,
    Mentor,
    Obstacle,
    Ally,
    Foil,
    ComicRelief,
    LoveInterest,
    Villain,
}

#[derive(Debug, Clone)]
pub struct PlotDevice {
    pub name: String,
    pub device_type: DeviceType,
    pub usage_conditions: Vec<String>,
    pub effects: Vec<PlotEffect>,
    pub overuse_penalty: f32,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    MacGuffin,
    DeusExMachina,
    RedHerring,
    ChekhovsGun,
    Foreshadowing,
    Flashback,
    PlotTwist,
    CliffHanger,
}

#[derive(Debug, Clone)]
pub struct PlotEffect {
    pub effect_type: String,
    pub magnitude: f32,
    pub duration: EffectDuration,
}

#[derive(Debug, Clone)]
pub enum EffectDuration {
    Temporary,
    Extended,
    Permanent,
}

#[derive(Debug, Clone)]
pub struct StorySeed {
    pub premise: String,
    pub genre_affinity: HashMap<Genre, f32>,
    pub required_elements: Vec<String>,
    pub character_suggestions: Vec<String>,
    pub setting_requirements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GenerationRule {
    pub name: String,
    pub condition: GenerationCondition,
    pub modification: StoryModification,
    pub priority: f32,
}

#[derive(Debug, Clone)]
pub enum GenerationCondition {
    GenreMatch(Genre),
    ElementPresent(String),
    CharacterCountThreshold(usize),
    ComplexityLevel(f32),
    DurationConstraint(u32),
}

#[derive(Debug, Clone)]
pub enum StoryModification {
    AddElement(String),
    RemoveElement(String),
    ModifyPacing(f32),
    AdjustComplexity(f32),
    ChangeGenre(Genre),
}

#[derive(Debug, Clone)]
pub struct NarrativePattern {
    pub name: String,
    pub sequence: Vec<PatternStep>,
    pub variations: Vec<PatternVariation>,
    pub effectiveness_rating: f32,
}

#[derive(Debug, Clone)]
pub struct PatternStep {
    pub step_type: StepType,
    pub description: String,
    pub requirements: Vec<String>,
    pub outcomes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum StepType {
    Setup,
    Inciting,
    Rising,
    Climax,
    Falling,
    Resolution,
}

#[derive(Debug, Clone)]
pub struct PatternVariation {
    pub name: String,
    pub modifications: Vec<StepModification>,
    pub suitability: Vec<Genre>,
}

#[derive(Debug, Clone)]
pub struct StepModification {
    pub step_index: usize,
    pub modification_type: ModificationType,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ModificationType {
    Replace,
    Insert,
    Remove,
    Modify,
    Reorder,
}

#[derive(Debug)]
pub struct GeneratedStory {
    pub title: String,
    pub premise: String,
    pub genre: Genre,
    pub characters: Vec<GeneratedCharacter>,
    pub plot_outline: Vec<PlotPoint>,
    pub themes: Vec<String>,
    pub estimated_duration: u32,
    pub complexity_score: f32,
}

#[derive(Debug)]
pub struct GeneratedCharacter {
    pub name: String,
    pub role: CharacterRole,
    pub archetype: String,
    pub background: String,
    pub motivations: Vec<String>,
    pub character_arc: CharacterArcOutline,
}

#[derive(Debug)]
pub struct CharacterArcOutline {
    pub starting_state: String,
    pub key_developments: Vec<String>,
    pub ending_state: String,
    pub internal_conflict: String,
}

#[derive(Debug)]
pub struct PlotPoint {
    pub sequence_number: usize,
    pub title: String,
    pub description: String,
    pub involved_characters: Vec<String>,
    pub plot_function: PlotFunction,
    pub emotional_beat: EmotionalBeat,
}

#[derive(Debug)]
pub enum PlotFunction {
    Exposition,
    IncitingIncident,
    PlotPoint1,
    Midpoint,
    PlotPoint2,
    Climax,
    Resolution,
}

#[derive(Debug)]
pub struct EmotionalBeat {
    pub primary_emotion: String,
    pub intensity: f32,
    pub character_impacts: HashMap<String, f32>,
}

impl StoryGenerator {
    pub fn new() -> Self {
        StoryGenerator {
            story_templates: HashMap::new(),
            character_archetypes: HashMap::new(),
            plot_devices: Vec::new(),
            story_seeds: Vec::new(),
            generation_rules: Vec::new(),
            narrative_patterns: HashMap::new(),
        }
    }

    pub fn generate_story(&mut self, constraints: &StoryConstraints) -> Result<GeneratedStory, String> {
        let template = self.select_template(constraints)?;
        let seed = self.select_story_seed(&template.genre)?;
        
        let characters = self.generate_characters(&template, constraints)?;
        let plot_outline = self.generate_plot_outline(&template, &characters, &seed)?;
        
        let story = GeneratedStory {
            title: self.generate_title(&template, &seed),
            premise: seed.premise.clone(),
            genre: template.genre.clone(),
            characters,
            plot_outline,
            themes: self.extract_themes(&template, &seed),
            estimated_duration: template.estimated_duration,
            complexity_score: self.calculate_complexity(&template),
        };
        
        Ok(story)
    }

    pub fn generate_quest_story(&mut self, quest_type: &str, characters: &[String]) -> Result<GeneratedStory, String> {
        let constraints = StoryConstraints {
            genre_preference: Some(Genre::Adventure),
            character_count: Some(characters.len()),
            max_duration: Some(120),
            complexity_level: Some(0.7),
            required_elements: vec![quest_type.to_string()],
            excluded_elements: Vec::new(),
        };
        
        self.generate_story(&constraints)
    }

    pub fn generate_character_story(&mut self, character_id: &str, story_type: &str) -> Result<GeneratedStory, String> {
        let constraints = StoryConstraints {
            genre_preference: self.infer_genre_from_story_type(story_type),
            character_count: Some(1),
            max_duration: Some(60),
            complexity_level: Some(0.5),
            required_elements: vec![character_id.to_string(), story_type.to_string()],
            excluded_elements: Vec::new(),
        };
        
        self.generate_story(&constraints)
    }

    pub fn generate_emergent_story(&mut self, world_events: &[String], active_characters: &[String]) -> Result<GeneratedStory, String> {
        let emergent_seed = self.create_emergent_seed(world_events, active_characters);
        
        let constraints = StoryConstraints {
            genre_preference: Some(Genre::Drama),
            character_count: Some(active_characters.len()),
            max_duration: Some(90),
            complexity_level: Some(0.6),
            required_elements: emergent_seed.required_elements.clone(),
            excluded_elements: Vec::new(),
        };
        
        self.generate_story(&constraints)
    }

    pub fn enhance_existing_story(&mut self, story: &mut GeneratedStory, enhancement_type: StoryEnhancement) {
        match enhancement_type {
            StoryEnhancement::AddSubplot => {
                self.add_subplot(story);
            }
            StoryEnhancement::DeepenCharacterArcs => {
                self.deepen_character_arcs(story);
            }
            StoryEnhancement::IncreaseTension => {
                self.increase_tension(story);
            }
            StoryEnhancement::AddTwist => {
                self.add_plot_twist(story);
            }
        }
    }

    fn select_template(&self, constraints: &StoryConstraints) -> Result<&StoryTemplate, String> {
        let suitable_templates: Vec<_> = self.story_templates
            .values()
            .filter(|template| self.template_matches_constraints(template, constraints))
            .collect();
            
        if suitable_templates.is_empty() {
            return Err("No suitable story templates found".to_string());
        }
        
        let mut rng = rand::thread_rng();
        let selected = suitable_templates[rng.gen_range(0..suitable_templates.len())];
        Ok(selected)
    }

    fn select_story_seed(&self, genre: &Genre) -> Result<&StorySeed, String> {
        let suitable_seeds: Vec<_> = self.story_seeds
            .iter()
            .filter(|seed| seed.genre_affinity.get(genre).unwrap_or(&0.0) > &0.5)
            .collect();
            
        if suitable_seeds.is_empty() {
            return Err("No suitable story seeds found".to_string());
        }
        
        let mut rng = rand::thread_rng();
        let selected = suitable_seeds[rng.gen_range(0..suitable_seeds.len())];
        Ok(selected)
    }

    fn generate_characters(&self, template: &StoryTemplate, constraints: &StoryConstraints) -> Result<Vec<GeneratedCharacter>, String> {
        let mut characters = Vec::new();
        
        for role in &template.character_roles {
            let archetype = self.select_archetype_for_role(role)?;
            let character = GeneratedCharacter {
                name: self.generate_character_name(&archetype),
                role: role.clone(),
                archetype: archetype.name.clone(),
                background: self.generate_character_background(&archetype),
                motivations: archetype.motivations.clone(),
                character_arc: self.generate_character_arc(&archetype, role),
            };
            characters.push(character);
        }
        
        Ok(characters)
    }

    fn generate_plot_outline(&self, template: &StoryTemplate, characters: &[GeneratedCharacter], seed: &StorySeed) -> Result<Vec<PlotPoint>, String> {
        let mut plot_points = Vec::new();
        let mut sequence_number = 0;
        
        for act in &template.structure.acts {
            for event in &act.key_events {
                let plot_point = PlotPoint {
                    sequence_number,
                    title: self.generate_plot_point_title(event),
                    description: self.generate_plot_point_description(event, characters, seed),
                    involved_characters: self.select_involved_characters(event, characters),
                    plot_function: self.determine_plot_function(sequence_number, &template.structure),
                    emotional_beat: self.generate_emotional_beat(event, characters),
                };
                plot_points.push(plot_point);
                sequence_number += 1;
            }
        }
        
        Ok(plot_points)
    }

    fn template_matches_constraints(&self, template: &StoryTemplate, constraints: &StoryConstraints) -> bool {
        if let Some(ref genre) = constraints.genre_preference {
            if std::mem::discriminant(&template.genre) != std::mem::discriminant(genre) {
                return false;
            }
        }
        
        if let Some(max_duration) = constraints.max_duration {
            if template.estimated_duration > max_duration {
                return false;
            }
        }
        
        if let Some(complexity) = constraints.complexity_level {
            if (template.complexity_level - complexity).abs() > 0.3 {
                return false;
            }
        }
        
        true
    }

    fn select_archetype_for_role(&self, role: &CharacterRole) -> Result<&CharacterArchetype, String> {
        let suitable_archetypes: Vec<_> = self.character_archetypes
            .values()
            .filter(|archetype| archetype.typical_roles.contains(&role.role_name))
            .collect();
            
        if suitable_archetypes.is_empty() {
            return Err(format!("No suitable archetypes found for role: {}", role.role_name));
        }
        
        let mut rng = rand::thread_rng();
        let selected = suitable_archetypes[rng.gen_range(0..suitable_archetypes.len())];
        Ok(selected)
    }

    fn generate_character_name(&self, archetype: &CharacterArchetype) -> String {
        format!("{}_Character_{}", archetype.name, rand::thread_rng().gen::<u32>() % 1000)
    }

    fn generate_character_background(&self, archetype: &CharacterArchetype) -> String {
        format!("A {} character with traits: {}", archetype.name, archetype.core_traits.join(", "))
    }

    fn generate_character_arc(&self, archetype: &CharacterArchetype, role: &CharacterRole) -> CharacterArcOutline {
        CharacterArcOutline {
            starting_state: "Established character state".to_string(),
            key_developments: vec!["Development 1".to_string(), "Development 2".to_string()],
            ending_state: "Transformed character state".to_string(),
            internal_conflict: archetype.core_traits.get(0).unwrap_or(&"Unknown".to_string()).clone(),
        }
    }

    fn generate_title(&self, template: &StoryTemplate, seed: &StorySeed) -> String {
        format!("Generated {} Story", template.name)
    }

    fn extract_themes(&self, template: &StoryTemplate, seed: &StorySeed) -> Vec<String> {
        vec!["Theme 1".to_string(), "Theme 2".to_string()]
    }

    fn calculate_complexity(&self, template: &StoryTemplate) -> f32 {
        template.complexity_level
    }

    fn generate_plot_point_title(&self, event: &str) -> String {
        format!("Plot Point: {}", event)
    }

    fn generate_plot_point_description(&self, event: &str, characters: &[GeneratedCharacter], seed: &StorySeed) -> String {
        format!("Description for event: {} involving characters", event)
    }

    fn select_involved_characters(&self, event: &str, characters: &[GeneratedCharacter]) -> Vec<String> {
        characters.iter().take(2).map(|c| c.name.clone()).collect()
    }

    fn determine_plot_function(&self, sequence: usize, structure: &StoryStructure) -> PlotFunction {
        match sequence {
            0 => PlotFunction::Exposition,
            1 => PlotFunction::IncitingIncident,
            _ => PlotFunction::Resolution,
        }
    }

    fn generate_emotional_beat(&self, event: &str, characters: &[GeneratedCharacter]) -> EmotionalBeat {
        EmotionalBeat {
            primary_emotion: "Tension".to_string(),
            intensity: 0.7,
            character_impacts: HashMap::new(),
        }
    }

    fn infer_genre_from_story_type(&self, story_type: &str) -> Option<Genre> {
        match story_type.to_lowercase().as_str() {
            "adventure" => Some(Genre::Adventure),
            "mystery" => Some(Genre::Mystery),
            "romance" => Some(Genre::Romance),
            _ => Some(Genre::Drama),
        }
    }

    fn create_emergent_seed(&self, world_events: &[String], active_characters: &[String]) -> StorySeed {
        StorySeed {
            premise: "An emergent story based on world events".to_string(),
            genre_affinity: [(Genre::Drama, 0.8)].iter().cloned().collect(),
            required_elements: world_events.to_vec(),
            character_suggestions: active_characters.to_vec(),
            setting_requirements: Vec::new(),
        }
    }

    fn add_subplot(&mut self, story: &mut GeneratedStory) {
        // Implementation for adding subplot
    }

    fn deepen_character_arcs(&mut self, story: &mut GeneratedStory) {
        // Implementation for deepening character arcs
    }

    fn increase_tension(&mut self, story: &mut GeneratedStory) {
        // Implementation for increasing tension
    }

    fn add_plot_twist(&mut self, story: &mut GeneratedStory) {
        // Implementation for adding plot twist
    }
}

#[derive(Debug)]
pub struct StoryConstraints {
    pub genre_preference: Option<Genre>,
    pub character_count: Option<usize>,
    pub max_duration: Option<u32>,
    pub complexity_level: Option<f32>,
    pub required_elements: Vec<String>,
    pub excluded_elements: Vec<String>,
}

#[derive(Debug)]
pub enum StoryEnhancement {
    AddSubplot,
    DeepenCharacterArcs,
    IncreaseTension,
    AddTwist,
}