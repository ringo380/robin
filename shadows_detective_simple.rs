use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Simplified Shadows of Doubt Style Detective Demo for Robin Engine
/// Demonstrates key detective gameplay concepts with voxel-based crime scenes

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Concrete,
    Brick,
    Glass,
    Wood,
    BloodStain,
    Fingerprint,
    FootprintMud,
    BrokenGlass,
    Door,
    SecurityCamera,
}

#[derive(Debug, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone)]
pub struct VoxelBlock {
    pub material: MaterialType,
    pub color: (u8, u8, u8),
    pub has_evidence: bool,
    pub evidence_id: Option<String>,
}

impl VoxelBlock {
    pub fn new(material: MaterialType) -> Self {
        let (color, has_evidence) = match material {
            MaterialType::Concrete => ((128, 128, 128), false),
            MaterialType::Brick => ((139, 69, 19), false),
            MaterialType::Glass => ((173, 216, 230), false),
            MaterialType::Wood => ((160, 82, 45), false),
            MaterialType::BloodStain => ((139, 0, 0), true),
            MaterialType::Fingerprint => ((105, 105, 105), true),
            MaterialType::FootprintMud => ((101, 67, 33), true),
            MaterialType::BrokenGlass => ((220, 220, 220), true),
            MaterialType::Door => ((139, 69, 19), false),
            MaterialType::SecurityCamera => ((96, 96, 96), false),
        };
        
        Self {
            material,
            color,
            has_evidence,
            evidence_id: None,
        }
    }
    
    pub fn with_evidence(mut self, evidence_id: String) -> Self {
        self.evidence_id = Some(evidence_id);
        self.has_evidence = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub position: Vector3,
    pub home_address: String,
    pub alibi: String,
    pub suspicious_level: f32,
    pub motive: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Evidence {
    pub id: String,
    pub description: String,
    pub location: Vector3,
    pub reliability: f32,
    pub points_to_suspect: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CrimeCase {
    pub id: String,
    pub victim: String,
    pub crime_scene: String,
    pub time_of_crime: String,
    pub suspects: Vec<String>,
    pub evidence: Vec<String>,
    pub solved: bool,
}

pub struct ShadowsDetectiveDemo {
    pub world_size: (i32, i32, i32),
    pub voxel_world: HashMap<(i32, i32, i32), VoxelBlock>,
    pub npcs: Vec<NPC>,
    pub evidence_items: Vec<Evidence>,
    pub case: CrimeCase,
    pub player_position: Vector3,
    pub investigation_progress: i32,
    pub current_phase: InvestigationPhase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvestigationPhase {
    CrimeSceneAnalysis,
    SuspectInterviews,
    EvidenceAnalysis,
    TheoryBuilding,
    CaseResolution,
}

impl ShadowsDetectiveDemo {
    pub fn new() -> Self {
        println!("🕵️ Initializing Shadows of Doubt Detective Demo...");
        
        let mut demo = Self {
            world_size: (32, 16, 32),
            voxel_world: HashMap::new(),
            npcs: Vec::new(),
            evidence_items: Vec::new(),
            case: CrimeCase {
                id: "murder_001".to_string(),
                victim: "Victoria Sterling".to_string(),
                crime_scene: "Apartment 4B".to_string(),
                time_of_crime: "Sunday 21:30".to_string(),
                suspects: vec![
                    "Marcus Black".to_string(),
                    "Elena Rodriguez".to_string(),
                    "Thomas Greene".to_string(),
                ],
                evidence: vec![
                    "bloody_knife".to_string(),
                    "fingerprints_door".to_string(),
                    "footprints_mud".to_string(),
                    "security_footage".to_string(),
                ],
                solved: false,
            },
            player_position: Vector3::new(16.0, 2.0, 16.0),
            investigation_progress: 0,
            current_phase: InvestigationPhase::CrimeSceneAnalysis,
        };
        
        demo.generate_crime_scene();
        demo.create_suspects();
        demo.place_evidence();
        
        demo
    }
    
    pub fn generate_crime_scene(&mut self) {
        println!("🏠 Generating crime scene apartment...");
        
        // Create apartment layout (10x10 room)
        let start_x = 10;
        let start_z = 10;
        let size = 10;
        
        // Floor
        for x in start_x..start_x + size {
            for z in start_z..start_z + size {
                self.voxel_world.insert(
                    (x, 0, z), 
                    VoxelBlock::new(MaterialType::Wood)
                );
            }
        }
        
        // Walls
        for x in start_x..start_x + size {
            for y in 1..4 {
                // Front and back walls
                self.voxel_world.insert(
                    (x, y, start_z), 
                    VoxelBlock::new(MaterialType::Brick)
                );
                self.voxel_world.insert(
                    (x, y, start_z + size - 1), 
                    VoxelBlock::new(MaterialType::Brick)
                );
            }
        }
        
        for z in start_z..start_z + size {
            for y in 1..4 {
                // Left and right walls
                self.voxel_world.insert(
                    (start_x, y, z), 
                    VoxelBlock::new(MaterialType::Brick)
                );
                self.voxel_world.insert(
                    (start_x + size - 1, y, z), 
                    VoxelBlock::new(MaterialType::Brick)
                );
            }
        }
        
        // Door
        self.voxel_world.insert(
            (start_x + size/2, 1, start_z), 
            VoxelBlock::new(MaterialType::Door)
        );
        
        // Windows
        self.voxel_world.insert(
            (start_x + 2, 2, start_z + size - 1), 
            VoxelBlock::new(MaterialType::Glass)
        );
        self.voxel_world.insert(
            (start_x + size - 3, 2, start_z + size - 1), 
            VoxelBlock::new(MaterialType::Glass)
        );
    }
    
    pub fn create_suspects(&mut self) {
        println!("👥 Creating suspect profiles...");
        
        let suspects = vec![
            NPC {
                id: "marcus_black".to_string(),
                name: "Marcus Black".to_string(),
                position: Vector3::new(5.0, 2.0, 5.0),
                home_address: "Apartment 2A".to_string(),
                alibi: "Was at the bar until midnight, has receipt".to_string(),
                suspicious_level: 0.7,
                motive: Some("Owed victim $50,000 from gambling debt".to_string()),
            },
            NPC {
                id: "elena_rodriguez".to_string(),
                name: "Elena Rodriguez".to_string(),
                position: Vector3::new(25.0, 2.0, 8.0),
                home_address: "Apartment 3C".to_string(),
                alibi: "Home alone, no witnesses".to_string(),
                suspicious_level: 0.4,
                motive: Some("Ex-girlfriend, recently broke up badly".to_string()),
            },
            NPC {
                id: "thomas_greene".to_string(),
                name: "Thomas Greene".to_string(),
                position: Vector3::new(8.0, 2.0, 25.0),
                home_address: "Apartment 1B".to_string(),
                alibi: "Working late at office, security logs confirm".to_string(),
                suspicious_level: 0.2,
                motive: None,
            },
        ];
        
        self.npcs = suspects;
    }
    
    pub fn place_evidence(&mut self) {
        println!("🔍 Placing evidence throughout crime scene...");
        
        let evidence_items = vec![
            Evidence {
                id: "bloody_knife".to_string(),
                description: "Kitchen knife with blood on blade, found under couch".to_string(),
                location: Vector3::new(13.0, 1.0, 15.0),
                reliability: 0.9,
                points_to_suspect: Some("marcus_black".to_string()),
            },
            Evidence {
                id: "fingerprints_door".to_string(),
                description: "Partial fingerprints on door handle".to_string(),
                location: Vector3::new(15.0, 1.0, 10.0),
                reliability: 0.6,
                points_to_suspect: Some("elena_rodriguez".to_string()),
            },
            Evidence {
                id: "footprints_mud".to_string(),
                description: "Muddy footprints size 11, leading from window".to_string(),
                location: Vector3::new(17.0, 1.0, 18.0),
                reliability: 0.8,
                points_to_suspect: Some("marcus_black".to_string()),
            },
            Evidence {
                id: "security_footage".to_string(),
                description: "Blurry figure seen entering building at 21:15".to_string(),
                location: Vector3::new(20.0, 3.0, 12.0),
                reliability: 0.4,
                points_to_suspect: None,
            },
        ];
        
        // Place evidence blocks in voxel world
        for evidence in &evidence_items {
            let pos = (
                evidence.location.x as i32,
                evidence.location.y as i32,
                evidence.location.z as i32,
            );
            
            let material = match evidence.id.as_str() {
                "bloody_knife" => MaterialType::BloodStain,
                "fingerprints_door" => MaterialType::Fingerprint,
                "footprints_mud" => MaterialType::FootprintMud,
                _ => MaterialType::BrokenGlass,
            };
            
            let block = VoxelBlock::new(material)
                .with_evidence(evidence.id.clone());
            
            self.voxel_world.insert(pos, block);
        }
        
        self.evidence_items = evidence_items;
    }
    
    pub fn run_investigation(&mut self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║  🕵️  SHADOWS OF DOUBT - DETECTIVE INVESTIGATION DEMO                        ║");
        println!("║  Voxel-based Crime Scene Investigation Simulation                           ║");
        println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");
        
        let start_time = Instant::now();
        let demo_duration = Duration::from_secs(40);
        
        let _phase_duration = Duration::from_secs(8);
        
        while start_time.elapsed() < demo_duration {
            let phase_elapsed = start_time.elapsed().as_secs() / 8;
            
            self.current_phase = match phase_elapsed {
                0 => InvestigationPhase::CrimeSceneAnalysis,
                1 => InvestigationPhase::SuspectInterviews,
                2 => InvestigationPhase::EvidenceAnalysis,
                3 => InvestigationPhase::TheoryBuilding,
                _ => InvestigationPhase::CaseResolution,
            };
            
            self.display_interface();
            
            match self.current_phase {
                InvestigationPhase::CrimeSceneAnalysis => self.analyze_crime_scene(),
                InvestigationPhase::SuspectInterviews => self.interview_suspects(),
                InvestigationPhase::EvidenceAnalysis => self.analyze_evidence(),
                InvestigationPhase::TheoryBuilding => self.build_theory(),
                InvestigationPhase::CaseResolution => self.resolve_case(),
            }
            
            self.investigation_progress += 20;
            
            std::thread::sleep(Duration::from_millis(2000));
        }
        
        self.display_final_results();
    }
    
    fn display_interface(&self) {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        
        println!("╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                     🕵️ DETECTIVE INVESTIGATION INTERFACE                     ║");
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
        
        println!("\n📋 CASE: {}", self.case.id);
        println!("   Victim: {}", self.case.victim);
        println!("   Crime Scene: {}", self.case.crime_scene);
        println!("   Time of Crime: {}", self.case.time_of_crime);
        println!("   Current Phase: {:?}", self.current_phase);
        println!("   Investigation Progress: {}%", self.investigation_progress.min(100));
        
        println!("\n🏠 CRIME SCENE LAYOUT (Voxel World)");
        self.display_crime_scene_map();
        
        println!("\n👤 SUSPECTS ({}):", self.npcs.len());
        for suspect in &self.npcs {
            println!("   • {} (Suspicion: {:.1}/1.0)", suspect.name, suspect.suspicious_level);
            if let Some(motive) = &suspect.motive {
                println!("     Motive: {}", motive);
            }
            println!("     Alibi: {}", suspect.alibi);
        }
        
        println!("\n🔍 EVIDENCE ({}):", self.evidence_items.len());
        for evidence in &self.evidence_items {
            println!("   • {} (Reliability: {:.1})", evidence.description, evidence.reliability);
            if let Some(suspect) = &evidence.points_to_suspect {
                let unknown = "Unknown".to_string();
                let suspect_name = self.npcs.iter()
                    .find(|npc| npc.id == *suspect)
                    .map(|npc| &npc.name)
                    .unwrap_or(&unknown);
                println!("     Points to: {}", suspect_name);
            }
        }
    }
    
    fn display_crime_scene_map(&self) {
        println!("   📍 Apartment Layout (Top View):");
        println!("   ┌──────────────────────────────┐");
        
        // Display a simple ASCII representation of the voxel world
        for z in 8..22 {
            print!("   │");
            for x in 8..22 {
                if let Some(block) = self.voxel_world.get(&(x, 1, z)) {
                    let symbol = match block.material {
                        MaterialType::Brick => '#',
                        MaterialType::Door => 'D',
                        MaterialType::Glass => 'W',
                        MaterialType::BloodStain => '!',
                        MaterialType::Fingerprint => 'F',
                        MaterialType::FootprintMud => 'M',
                        MaterialType::BrokenGlass => 'G',
                        MaterialType::Wood => '.',
                        _ => ' ',
                    };
                    print!("{}", symbol);
                } else if let Some(_block) = self.voxel_world.get(&(x, 0, z)) {
                    print!(".");
                } else {
                    print!(" ");
                }
            }
            println!("│");
        }
        
        println!("   └──────────────────────────────┘");
        println!("   Legend: # Wall, D Door, W Window, ! Blood, F Fingerprint, M Footprint");
    }
    
    fn analyze_crime_scene(&mut self) {
        println!("\n🔍 CRIME SCENE ANALYSIS");
        println!("   Examining voxel blocks for evidence...");
        
        let evidence_blocks: Vec<_> = self.voxel_world
            .iter()
            .filter(|(_, block)| block.has_evidence)
            .collect();
        
        println!("   Found {} evidence blocks in crime scene:", evidence_blocks.len());
        
        for ((x, y, z), block) in evidence_blocks {
            println!("   📍 Evidence at ({}, {}, {}): {:?}", x, y, z, block.material);
            if let Some(evidence_id) = &block.evidence_id {
                if let Some(evidence) = self.evidence_items.iter().find(|e| e.id == *evidence_id) {
                    println!("      Detail: {}", evidence.description);
                }
            }
        }
        
        println!("   🔬 Forensic analysis reveals multiple perpetrator traces");
        println!("   💭 Initial assessment: Planned attack, perpetrator familiar with victim");
    }
    
    fn interview_suspects(&mut self) {
        println!("\n👥 SUSPECT INTERVIEWS");
        
        for suspect in &self.npcs {
            println!("   🗣️ Interviewing: {}", suspect.name);
            println!("      Location: {}", suspect.home_address);
            println!("      Alibi: {}", suspect.alibi);
            
            // Simulate interview insights
            match suspect.id.as_str() {
                "marcus_black" => {
                    println!("      🚨 Shows signs of nervousness, inconsistent story");
                    println!("      💰 Financial motive confirmed - gambling debt");
                    println!("      🍺 Bar receipt time doesn't match witness accounts");
                },
                "elena_rodriguez" => {
                    println!("      😢 Emotional about victim, confirms recent breakup");
                    println!("      🏠 No solid alibi, was home alone");
                    println!("      💔 Claims victim was threatening to expose secrets");
                },
                "thomas_greene" => {
                    println!("      😐 Calm and cooperative, provides detailed timeline");
                    println!("      🏢 Office security confirms presence until 22:30");
                    println!("      🤝 No apparent motive, friendly relationship with victim");
                },
                _ => {}
            }
            println!();
        }
    }
    
    fn analyze_evidence(&mut self) {
        println!("\n🔬 FORENSIC EVIDENCE ANALYSIS");
        
        for evidence in &self.evidence_items {
            println!("   🧪 Analyzing: {}", evidence.description);
            
            match evidence.id.as_str() {
                "bloody_knife" => {
                    println!("      🩸 Blood type matches victim");
                    println!("      👤 Fingerprints found - running through database");
                    println!("      📊 85% match to Marcus Black");
                },
                "fingerprints_door" => {
                    println!("      👆 Partial print recovered from door handle");
                    println!("      🔍 Quality: Poor due to smudging");
                    println!("      ❓ Inconclusive - could match multiple suspects");
                },
                "footprints_mud" => {
                    println!("      👟 Size 11 boot print, fresh mud");
                    println!("      🌧️ Mud composition matches park soil");
                    println!("      📏 Gait analysis suggests male, ~180cm height");
                },
                "security_footage" => {
                    println!("      📹 Enhancement reveals partial face");
                    println!("      👤 Build consistent with Marcus Black");
                    println!("      ⏰ Timestamp: 21:47 PM - after crime window");
                },
                _ => {}
            }
            println!("      Reliability Score: {:.1}/1.0", evidence.reliability);
            println!();
        }
    }
    
    fn build_theory(&mut self) {
        println!("\n🧠 BUILDING CASE THEORY");
        
        println!("   📝 Theory 1: Marcus Black as Primary Suspect");
        println!("      ✅ Strong financial motive ($50,000 gambling debt)");
        println!("      ✅ Physical evidence (fingerprints on murder weapon)");
        println!("      ✅ Size and build match footprint evidence");
        println!("      ❌ Alibi partially corroborated by bar receipt");
        println!("      📊 Probability: 75%");
        
        println!("\n   📝 Theory 2: Elena Rodriguez Crime of Passion");
        println!("      ✅ Emotional motive (recent bitter breakup)");
        println!("      ✅ Access to victim's apartment");
        println!("      ✅ No solid alibi for time of crime");
        println!("      ❌ Physical evidence doesn't strongly support");
        println!("      📊 Probability: 35%");
        
        println!("\n   📝 Theory 3: Unknown Third Party");
        println!("      ❓ Possible robbery gone wrong");
        println!("      ❓ Evidence could be planted");
        println!("      ❌ No signs of forced entry or theft");
        println!("      📊 Probability: 15%");
        
        println!("\n   🎯 RECOMMENDED ACTION: Focus investigation on Marcus Black");
        println!("      🔍 Verify bar alibi with additional witnesses");
        println!("      🧪 Rush DNA analysis on murder weapon");
        println!("      📱 Subpoena phone records for timeline verification");
    }
    
    fn resolve_case(&mut self) {
        println!("\n⚖️ CASE RESOLUTION");
        
        // Calculate evidence strength
        let total_evidence_strength: f32 = self.evidence_items
            .iter()
            .map(|e| e.reliability)
            .sum();
        
        let marcus_evidence_strength: f32 = self.evidence_items
            .iter()
            .filter(|e| e.points_to_suspect.as_ref() == Some(&"marcus_black".to_string()))
            .map(|e| e.reliability)
            .sum();
        
        println!("   📊 FINAL EVIDENCE ASSESSMENT:");
        println!("      Total Evidence Strength: {:.1}", total_evidence_strength);
        println!("      Evidence Against Marcus Black: {:.1}", marcus_evidence_strength);
        println!("      Case Strength: {:.0}%", (marcus_evidence_strength / total_evidence_strength * 100.0));
        
        if marcus_evidence_strength > 1.5 {
            println!("\n   🚨 ARREST WARRANT ISSUED");
            println!("      Suspect: Marcus Black");
            println!("      Charges: First-degree murder");
            println!("      Evidence: Murder weapon with fingerprints, weak alibi, clear motive");
            
            self.case.solved = true;
            
            println!("\n   ⚖️ CASE OUTCOME PREDICTION:");
            println!("      Conviction Probability: 85%");
            println!("      Expected Sentence: 25 years to life");
            println!("      Case Status: SOLVED");
        } else {
            println!("\n   ❌ INSUFFICIENT EVIDENCE");
            println!("      Case remains open for further investigation");
            println!("      Recommended: Additional forensic analysis");
        }
    }
    
    fn display_final_results(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                    🏆 INVESTIGATION COMPLETE - FINAL REPORT                  ║");
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
        
        println!("\n📋 CASE SUMMARY");
        println!("   Case ID: {}", self.case.id);
        println!("   Victim: {}", self.case.victim);
        println!("   Crime Scene: {}", self.case.crime_scene);
        println!("   Status: {}", if self.case.solved { "SOLVED ✅" } else { "UNSOLVED ❌" });
        
        println!("\n🔍 INVESTIGATION STATISTICS");
        println!("   Evidence Items Collected: {}", self.evidence_items.len());
        println!("   Suspects Interviewed: {}", self.npcs.len());
        println!("   Crime Scene Blocks Analyzed: {}", 
            self.voxel_world.values().filter(|b| b.has_evidence).count());
        println!("   Investigation Progress: {}%", self.investigation_progress.min(100));
        
        println!("\n🏗️ VOXEL WORLD STATISTICS");
        println!("   Total Voxel Blocks: {}", self.voxel_world.len());
        println!("   Evidence Blocks: {}", 
            self.voxel_world.values().filter(|b| b.has_evidence).count());
        println!("   World Dimensions: {}×{}×{}", 
            self.world_size.0, self.world_size.1, self.world_size.2);
        
        // Count block types
        let mut material_counts = HashMap::new();
        for block in self.voxel_world.values() {
            *material_counts.entry(format!("{:?}", block.material)).or_insert(0) += 1;
        }
        
        println!("\n🧱 MATERIAL DISTRIBUTION");
        for (material, count) in material_counts {
            println!("   {}: {}", material, count);
        }
        
        println!("\n🕵️ DETECTIVE PERFORMANCE");
        let performance_score = if self.case.solved { 90.0 } else { 60.0 };
        let grade = match performance_score as i32 {
            90..=100 => "A+ (Master Detective)",
            80..=89 => "A (Expert Investigator)",
            70..=79 => "B (Skilled Detective)",
            60..=69 => "C (Competent Officer)",
            _ => "D (Needs Training)",
        };
        
        println!("   Performance Score: {:.0}/100", performance_score);
        println!("   Investigation Grade: {}", grade);
        println!("   Time to Resolution: 40 seconds");
        
        println!("\n✨ SHADOWS OF DOUBT FEATURES DEMONSTRATED");
        println!("   ✅ Voxel-based crime scene construction");
        println!("   ✅ Evidence placement and discovery system");
        println!("   ✅ Multi-suspect investigation with motives and alibis");
        println!("   ✅ Forensic analysis and reliability scoring");
        println!("   ✅ Theory building and case resolution logic");
        println!("   ✅ Real-time investigation progress tracking");
        println!("   ✅ Detective performance evaluation system");
        
        println!("\n🚀 Robin Engine successfully demonstrated detective gameplay!");
        println!("   Framework ready for full Shadows of Doubt style game development.");
    }
}

pub fn main() {
    println!("🕵️ Starting Shadows of Doubt Detective Demo for Robin Engine...\n");
    
    let mut demo = ShadowsDetectiveDemo::new();
    demo.run_investigation();
    
    println!("\n🎉 Detective demo completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_initialization() {
        let demo = ShadowsDetectiveDemo::new();
        
        assert_eq!(demo.npcs.len(), 3);
        assert_eq!(demo.evidence_items.len(), 4);
        assert!(!demo.case.victim.is_empty());
        assert_eq!(demo.current_phase, InvestigationPhase::CrimeSceneAnalysis);
    }
    
    #[test]
    fn test_crime_scene_generation() {
        let demo = ShadowsDetectiveDemo::new();
        
        // Check that crime scene has walls
        let has_walls = demo.voxel_world.values().any(|b| b.material == MaterialType::Brick);
        assert!(has_walls);
        
        // Check that evidence blocks exist
        let has_evidence = demo.voxel_world.values().any(|b| b.has_evidence);
        assert!(has_evidence);
    }
    
    #[test]
    fn test_suspect_profiles() {
        let demo = ShadowsDetectiveDemo::new();
        
        assert_eq!(demo.npcs.len(), 3);
        
        // Check Marcus Black has high suspicion
        let marcus = demo.npcs.iter().find(|npc| npc.name == "Marcus Black").unwrap();
        assert!(marcus.suspicious_level > 0.5);
        assert!(marcus.motive.is_some());
    }
    
    #[test]
    fn test_evidence_system() {
        let demo = ShadowsDetectiveDemo::new();
        
        assert_eq!(demo.evidence_items.len(), 4);
        
        // Check evidence has reliability scores
        for evidence in &demo.evidence_items {
            assert!(evidence.reliability > 0.0 && evidence.reliability <= 1.0);
        }
    }
}