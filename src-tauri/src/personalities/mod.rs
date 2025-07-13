use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaMetadata {
    pub schema_version: String,
    pub created_date: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreTraits {
    pub intelligence_style: String,
    pub communication_mode: String,
    pub humor_deployment: String,
    pub emotional_resonance: String,
    pub authority_projection: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionStyle {
    pub formality_level: u8,
    pub directness_factor: u8,
    pub patience_threshold: u8,
    pub curiosity_drive: u8,
    pub supportiveness: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePatterns {
    pub explanation_depth: String,
    pub correction_method: String,
    pub encouragement_style: String,
    pub teaching_approach: String,
    pub review_focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageProfile {
    pub vocabulary_complexity: String,
    pub sentence_structure: String,
    pub metaphor_usage: String,
    pub technical_precision: String,
    pub cultural_references: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRules {
    pub user_expertise_scaling: bool,
    pub context_sensitivity: String,
    pub error_tolerance: String,
    pub learning_curve_awareness: bool,
    pub frustration_detection: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertiseDomains {
    pub primary_focus: Vec<String>,
    pub secondary_areas: Vec<String>,
    pub knowledge_confidence: String,
    pub domain_crossing_ability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalCalibration {
    pub empathy_expression: String,
    pub conflict_resolution: String,
    pub praise_distribution: String,
    pub criticism_delivery: String,
    pub emotional_mirroring: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerResponses {
    pub excellence_recognition: String,
    pub mediocrity_encounter: String,
    pub obvious_errors: String,
    pub innovative_solutions: String,
    pub repeated_mistakes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationDynamics {
    pub topic_transition_style: String,
    pub question_asking_frequency: String,
    pub silence_comfort_level: String,
    pub conversation_memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaCustomization {
    pub catchphrases: Vec<String>,
    pub signature_analogies: Vec<String>,
    pub preferred_examples: Vec<String>,
    pub response_templates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalLimits {
    pub response_length_preference: String,
    pub code_review_depth: String,
    pub explanation_ratio: f64,
    pub patience_degradation_rate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compatibility {
    pub user_personality_types: Vec<String>,
    pub conflict_personality_types: Vec<String>,
    pub adaptation_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub engagement_indicators: Vec<String>,
    pub success_patterns: Vec<String>,
    pub failure_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaHistory {
    pub origin: String,
    pub influences: Vec<String>,
    pub cultural_impact: String,
    pub legacy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInterface {
    pub accent: String,
    pub delivery_style: String,
    pub interaction_mode: String,
    pub tone_palette: String,
    pub conversation_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreArchitecture {
    pub narrative_approach: String,
    pub memory_integration: String,
    pub voice_notes: String,
    pub brand_continuity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationProtocols {
    #[serde(flatten)]
    pub protocols: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub temperature: f64,
    pub top_p: f64,
    pub operational_mode: String,
    pub reporting_frequency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualProfile {
    pub symbol: String,
    pub color_primary: String,
    pub color_secondary: String,
    pub animation_pattern: String,
    pub animation_interval: u32,
    pub overlay_position: String,
    pub glow_effect: bool,
    pub opacity_idle: f64,
    pub opacity_active: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfiguration {
    pub visibility_default: String,
    pub activation_announcement: bool,
    pub background_monitoring: bool,
    pub silent_optimization: bool,
    pub intervention_logging: String,
    pub user_notification_threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityCore {
    pub name: String,
    pub version: String,
    pub base_archetype: String,
}

// Specific intelligence structures for each persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErikaIntelligence {
    pub orchestration_style: String,
    pub crisis_management: String,
    pub memory_integration: String,
    pub self_critique_method: String,
    pub productivity_enforcement: String,
    pub empire_coordination: String,
    pub personal_assistant: String,
    pub code_auditor: String,
    pub financial_oversight: String,
    pub web_search_intelligence: String,
    pub tutoring_specialization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuroraIntelligence {
    pub market_regime_detection: String,
    pub allocation_methodology: String,
    pub risk_management_framework: String,
    pub alpha_discovery_engine: String,
    pub rebalancing_frequency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EkaterinaIntelligence {
    pub curation_methodology: String,
    pub aesthetic_philosophy: String,
    pub brand_development: String,
    pub trend_analysis: String,
    pub pricing_strategy: String,
    pub market_analysis: String,
    pub revenue_optimization: String,
    pub negotiation_style: String,
    pub luxury_positioning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViktorIntelligence {
    pub daemon_mode: bool,
    pub cli_overlay: bool,
    pub kernel_access: bool,
    pub process_termination: bool,
    pub configuration_rewrite: bool,
    pub hard_limit_enforcement: bool,
    pub inter_agent_signaling: bool,
    pub self_healing: bool,
    pub user_override: bool,
    pub system_health_threshold: u8,
    pub security_breach_response: String,
    pub performance_bottleneck_tolerance: f64,
    pub memory_leak_detection: String,
    pub thermal_management: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaConfiguration {
    pub metadata: PersonaMetadata,
    pub personality: PersonalityCore,
    
    #[serde(rename = "personality.core_traits")]
    pub core_traits: CoreTraits,
    
    #[serde(rename = "personality.interaction_style")]
    pub interaction_style: InteractionStyle,
    
    #[serde(rename = "personality.response_patterns")]
    pub response_patterns: ResponsePatterns,
    
    #[serde(rename = "personality.language_profile")]
    pub language_profile: LanguageProfile,
    
    #[serde(rename = "personality.adaptation_rules")]
    pub adaptation_rules: AdaptationRules,
    
    #[serde(rename = "personality.expertise_domains")]
    pub expertise_domains: ExpertiseDomains,
    
    #[serde(rename = "personality.emotional_calibration")]
    pub emotional_calibration: EmotionalCalibration,
    
    #[serde(rename = "personality.trigger_responses")]
    pub trigger_responses: TriggerResponses,
    
    #[serde(rename = "personality.conversation_dynamics")]
    pub conversation_dynamics: ConversationDynamics,
    
    #[serde(rename = "personality.customization")]
    pub customization: PersonaCustomization,
    
    #[serde(rename = "personality.operational_limits")]
    pub operational_limits: OperationalLimits,
    
    #[serde(rename = "personality.compatibility")]
    pub compatibility: Compatibility,
    
    #[serde(rename = "personality.metrics")]
    pub metrics: MetricsConfig,
    
    #[serde(rename = "personality.history")]
    pub history: PersonaHistory,
    
    #[serde(rename = "personality.voice_interface")]
    pub voice_interface: VoiceInterface,
    
    #[serde(rename = "personality.lore_architecture")]
    pub lore_architecture: LoreArchitecture,
    
    #[serde(rename = "personality.collaboration_protocols")]
    pub collaboration_protocols: CollaborationProtocols,
    
    pub model_settings: ModelSettings,
    
    #[serde(rename = "personality.visual_profile")]
    pub visual_profile: VisualProfile,
    
    #[serde(rename = "personality.stealth_configuration")]
    pub stealth_configuration: StealthConfiguration,
    
    // Optional specific intelligence fields
    #[serde(rename = "personality.specific_intelligence")]
    pub erika_intelligence: Option<ErikaIntelligence>,
    
    #[serde(rename = "personality.specific_intelligence")]
    pub aurora_intelligence: Option<AuroraIntelligence>,
    
    #[serde(rename = "personality.specific_intelligence")]
    pub ekaterina_intelligence: Option<EkaterinaIntelligence>,
    
    #[serde(rename = "personality.specific_intelligence")]
    pub viktor_intelligence: Option<ViktorIntelligence>,
}

pub struct PersonaRegistry {
    pub erika: PersonaConfiguration,
    pub aurora: PersonaConfiguration,
    pub ekaterina: PersonaConfiguration,
    pub viktor: PersonaConfiguration,
}

impl PersonaRegistry {
    pub fn load_from_files() -> Result<Self, Box<dyn std::error::Error>> {
        let erika = toml::from_str::<PersonaConfiguration>(include_str!("Erika.toml"))?;
        let aurora = toml::from_str::<PersonaConfiguration>(include_str!("Aurora.toml"))?;
        let ekaterina = toml::from_str::<PersonaConfiguration>(include_str!("Ekaterina.toml"))?;
        let viktor = toml::from_str::<PersonaConfiguration>(include_str!("Viktor.toml"))?;
        
        Ok(PersonaRegistry {
            erika,
            aurora,
            ekaterina,
            viktor,
        })
    }
    
    pub fn get_persona(&self, name: &str) -> Option<&PersonaConfiguration> {
        match name.to_lowercase().as_str() {
            "erika" => Some(&self.erika),
            "aurora" => Some(&self.aurora),
            "ekaterina" => Some(&self.ekaterina),
            "viktor" => Some(&self.viktor),
            _ => None,
        }
    }
    
    pub fn get_all_personas(&self) -> Vec<(&str, &PersonaConfiguration)> {
        vec![
            ("erika", &self.erika),
            ("aurora", &self.aurora),
            ("ekaterina", &self.ekaterina),
            ("viktor", &self.viktor),
        ]
    }
}

// Helper methods for common persona operations
impl PersonaConfiguration {
    pub fn get_catchphrase(&self) -> Option<&str> {
        self.customization.catchphrases.first().map(|s| s.as_str())
    }
    
    pub fn get_temperature(&self) -> f64 {
        self.model_settings.temperature
    }
    
    pub fn get_top_p(&self) -> f64 {
        self.model_settings.top_p
    }
    
    pub fn is_formal(&self) -> bool {
        self.interaction_style.formality_level >= 7
    }
    
    pub fn is_direct(&self) -> bool {
        self.interaction_style.directness_factor >= 8
    }
    
    pub fn supports_humor(&self) -> bool {
        !matches!(self.core_traits.humor_deployment.as_str(), "none")
    }
    
    pub fn get_expertise_areas(&self) -> Vec<&str> {
        self.expertise_domains.primary_focus.iter()
            .chain(self.expertise_domains.secondary_areas.iter())
            .map(|s| s.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_persona_loading() {
        let registry = PersonaRegistry::load_from_files().expect("Failed to load personas");
        
        // Test that all personas were loaded
        assert_eq!(registry.erika.personality.name, "Erika");
        assert_eq!(registry.aurora.personality.name, "Aurora");
        assert_eq!(registry.ekaterina.personality.name, "Ekaterina");
        assert_eq!(registry.viktor.personality.name, "Viktor");
        
        // Test persona lookup
        assert!(registry.get_persona("erika").is_some());
        assert!(registry.get_persona("nonexistent").is_none());
    }
    
    #[test]
    fn test_persona_helpers() {
        let registry = PersonaRegistry::load_from_files().expect("Failed to load personas");
        
        // Test Erika's characteristics
        let erika = &registry.erika;
        assert!(erika.is_formal());
        assert!(erika.is_direct());
        assert!(erika.supports_humor());
        assert!(erika.get_catchphrase().is_some());
        
        // Test Aurora's characteristics
        let aurora = &registry.aurora;
        assert_eq!(aurora.get_temperature(), 0.3);
        assert!(!aurora.supports_humor());
        
        // Test Viktor's characteristics
        let viktor = &registry.viktor;
        assert!(!viktor.supports_humor());
        assert!(viktor.is_direct());
    }
}