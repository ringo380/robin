//! Unit tests for enhanced skill tree system

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::save_system::PlayerData;

    #[test]
    fn test_enhanced_skill_manager_creation() {
        let skill_manager = EnhancedSkillManager::new();

        // Verify all specialization paths are initialized
        assert_eq!(skill_manager.skill_trees.len(), 4);
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Engineer));
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Artist));
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Explorer));
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Researcher));

        // Verify initial talent points
        assert_eq!(skill_manager.talent_points.available, 0);
        assert_eq!(skill_manager.talent_points.spent, 0);
        assert_eq!(skill_manager.talent_points.earned, 0);
    }

    #[test]
    fn test_talent_point_award() {
        let mut skill_manager = EnhancedSkillManager::new();

        // Award talent points for engineering skill
        skill_manager.award_talent_points(BuildingSkill::Engineering, 2);

        // Engineering gets bonus points, so should be 2 base + 1 bonus = 3
        assert_eq!(skill_manager.talent_points.available, 3);
        assert_eq!(skill_manager.talent_points.earned, 3);

        // Award points for construction (no bonus)
        skill_manager.award_talent_points(BuildingSkill::Construction, 1);

        assert_eq!(skill_manager.talent_points.available, 4);
        assert_eq!(skill_manager.talent_points.earned, 4);
    }

    #[test]
    fn test_talent_point_allocation() {
        let mut skill_manager = EnhancedSkillManager::new();
        let mut player_data = PlayerData::new("test_player");

        // Award some talent points
        skill_manager.award_talent_points(BuildingSkill::Engineering, 1);
        assert_eq!(skill_manager.talent_points.available, 1);

        // Allocate to engineer foundation
        let result = skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "engineer_foundation",
            &mut player_data
        ).unwrap();

        assert_eq!(result.node_id, "engineer_foundation");
        assert_eq!(result.points_allocated, 1);
        assert_eq!(skill_manager.talent_points.available, 0);
        assert_eq!(skill_manager.talent_points.spent, 1);
    }

    #[test]
    fn test_specialization_summary() {
        let mut skill_manager = EnhancedSkillManager::new();
        let mut player_data = PlayerData::new("test_player");

        // Award and allocate some points
        skill_manager.award_talent_points(BuildingSkill::Engineering, 3);
        skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "engineer_foundation",
            &mut player_data
        ).unwrap();

        let summary = skill_manager.get_specialization_summary();

        assert_eq!(summary.engineer_points, 1);
        assert_eq!(summary.artist_points, 0);
        assert_eq!(summary.talent_points.available, 2);
        assert_eq!(summary.talent_points.spent, 1);
        assert_eq!(summary.primary_specialization, None); // Need 5+ points for specialization
    }

    #[test]
    fn test_respec_functionality() {
        let mut skill_manager = EnhancedSkillManager::new();
        let mut player_data = PlayerData::new("test_player");

        // Award and spend talent points
        skill_manager.award_talent_points(BuildingSkill::Engineering, 2);
        skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "engineer_foundation",
            &mut player_data
        ).unwrap();

        assert_eq!(skill_manager.talent_points.available, 1);
        assert_eq!(skill_manager.talent_points.spent, 1);

        // Reset specializations
        let refunded = skill_manager.reset_specializations(&mut player_data).unwrap();

        assert_eq!(refunded, 1);
        assert_eq!(skill_manager.talent_points.available, 2);
        assert_eq!(skill_manager.talent_points.spent, 0);

        let summary = skill_manager.get_specialization_summary();
        assert_eq!(summary.engineer_points, 0);
    }

    #[test]
    fn test_prerequisite_checking() {
        let mut skill_manager = EnhancedSkillManager::new();
        let mut player_data = PlayerData::new("test_player");

        // Award talent points
        skill_manager.award_talent_points(BuildingSkill::Engineering, 5);

        // Try to allocate to a node with prerequisites without meeting them
        let result = skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "advanced_automation",
            &mut player_data
        );

        // Should fail because prerequisites not met
        assert!(result.is_err());

        // First allocate to prerequisites
        skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "engineer_foundation",
            &mut player_data
        ).unwrap();

        skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "logic_circuits",
            &mut player_data
        ).unwrap();

        skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "automation_basics",
            &mut player_data
        ).unwrap();

        // Now should be able to allocate to advanced_automation
        let result = skill_manager.allocate_talent_point(
            SpecializationPath::Engineer,
            "advanced_automation",
            &mut player_data
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_apple_silicon_optimized_calculation() {
        let skill_manager = EnhancedSkillManager::new();

        // Create a skill node with complex bonuses to test optimization paths
        let complex_node = SkillNode {
            id: "test_complex".to_string(),
            name: "Complex Test Node".to_string(),
            description: "Testing optimization".to_string(),
            max_points: 5,
            current_points: 3,
            prerequisites: Vec::new(),
            unlocks: Vec::new(),
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::SpeedIncrease(0.1),
                    per_point: true,
                },
                SkillBonus {
                    bonus_type: BonusType::QualityBonus(0.05),
                    per_point: true,
                },
                SkillBonus {
                    bonus_type: BonusType::CostReduction(0.15),
                    per_point: false,
                },
            ],
            tier: 3,
        };

        let bonuses = skill_manager.calculate_node_bonuses(&complex_node);

        // Should have 3 bonuses applied
        assert_eq!(bonuses.len(), 3);

        // Check per-point bonuses are multiplied correctly
        let speed_bonus = bonuses.iter().find(|b| matches!(b.bonus_type, BonusType::SpeedIncrease(_))).unwrap();
        assert_eq!(speed_bonus.strength, 3.0); // 3 points allocated

        let quality_bonus = bonuses.iter().find(|b| matches!(b.bonus_type, BonusType::QualityBonus(_))).unwrap();
        assert_eq!(quality_bonus.strength, 3.0); // 3 points allocated

        let cost_bonus = bonuses.iter().find(|b| matches!(b.bonus_type, BonusType::CostReduction(_))).unwrap();
        assert_eq!(cost_bonus.strength, 1.0); // Not per-point
    }
}