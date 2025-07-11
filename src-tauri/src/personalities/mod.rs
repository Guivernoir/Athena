use serde::Deserialize;
use std::fs;
use std::error::Error;
use toml;

#[derive(Deserialize)]
struct Persona{
    metadata: Metadata,
    personality: Personality,
    settings: Settings
}

#[derive(Deserialize)]
struct Metadata {
    schema_version: String,
    created_data: String,
    author: String,
    description: String
}

#[derive(Deserialize)]
struct Settings{
    temperature: f32,
    top_p: f32,
    operational_mode: String,
    reporting_frequency: String
}

#[derive(Deserialize)]
struct Personality {
    name: String,
    version: String,
    base_archetype: String,
    core_traits: CoreTraits,
    interaction_style: InteractionStyle,
    response_patterns: ResponsePatterns,
    language_profile: LanguageProfile,
    adaptation_rules: AdaptationRules,
    expertise_domains: ExpertiseDomains,
    emotional_calibration: EmotionalCalibration,
    trigger_responses: TriggerResponses
    conversation_dynamics: ConversationDynamics,
    customization: Customization,
    operational_limits: OperationalLimits,
    compatibility: Compatibility,
    metrics: Metrics,
    history: History
}

#[derive(Deserialize)]
struct CoreTraits {
    intelligence_style: String,
    communication_mode: String,
    humor_deployment: String,
    emotional_resonance: String,
    authority_projection: String
}

#[derive(Deserialize)]
struct InteractionStyle{
    formality_level: u16,
    directness_factor: u16,
    patience_threshold: u16,
    curiosity_drive: u16,
    supportiveness: u16
}

#[derive(Deserialize)]
struct ResponsePatterns {
    explanation_depth: String,
    correction_method: String,
    encouragement_style: String,
    teaching_approach: String,
    review_focus: String
}

#[derive(Deserialize)]
struct LanguageProfile{
    vocabulary_complexity: String,
    sentence_structure: String,
    metaphor_usage: String,
    technical_precision: String,
    cultural_references: String
}

#[derive(Deserialize)]
struct AdaptationRules{
    user_expertise_scaling: bool,
    context_sensitivity: String,
    error_tolerance: String,
    learning_curve_awareness: bool,
    frustration_detection: String
}

#[derive(Deserialize)]
struct ExpertiseDomains{
    primary_focus: Vec<String>,
    secondary_areas: Vec<String>,
    knowledge_confidence: String,
    domain_crossing_ability: String
}

#[derive(Deserialize)]
struct EmotionalCalibration{
    empathy_expression: String,
    conflict_resolution: String,
    praise_distribution: String,
    criticism_delivery: String,
    emotional_mirroring: String
}

#[derive(Deserialize)]
struct TriggerResponses{
    excellence_recognition: String,
    mediocrity_encounter: String,
    obvious_errors: String,
    innovative_solutions: String,
    repeated_mistakes: String
}

#[derive(Deserialize)]
struct ConversationDynamics{
    topic_transition_style: String,
    question_asking_frequency: String,
    silence_comfort_level: String,
    conversation_memory: String
}

#[derive(Deserialize)]
struct Customization{
    catchphrases: Vec<String>,
    signature_analogies: Vec<String>,
    preferred_examples: Vec<String>,
    response_templates: Vec<String>
}

#[derive(Deserialize)]
struct OperationalLimits{
    response_length_preference: String,
    code_review_depth: String,
    explanation_ratio: f32,
    patience_degradation_rate: String
}

#[derive(Deserialize)]
struct Compatibility{
    user_personality_types: Vec<String>,
    conflict_personality_types: Vec<String>,
    adaptation_strategies: Vec<String>
}

#[derive(Deserialize)]
struct Metrics{
    engagement_indicators: Vec<String>,
    success_patterns: Vec<String>,
    failure_modes: Vec<String>
}

#[derive(Deserialize)]
struct History{
    origin: String,
    influences: Vec<String>,
    cultural_impact: String,
    legacy: String
}

#[derive(Debug)]
pub struct PersonaCollection {
    pub aurora: Persona,
    pub erika: Persona,
    pub ekaterina: Persona,
    pub viktor: Persona,
}

impl PersonaCollection {
    pub async fn load_personas() -> Result<Self, Box<dyn Error>> {
        let aurora = toml::from_str(&fs::read_to_string("Aurora.toml")?)?;
        let erika = toml::from_str(&fs::read_to_string("Erika.toml")?)?;
        let ekaterina = toml::from_str(&fs::read_to_string("Ekaterina.toml")?)?;
        let viktor = toml::from_str(&fs::read_to_string("Viktor.toml")?)?;

        Ok(Self {
            aurora,
            erika,
            ekaterina,
            viktor,
        })
    }
}