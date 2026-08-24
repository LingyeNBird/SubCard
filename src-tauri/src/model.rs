use serde::{Deserialize, Serialize};
use std::sync::Mutex;

fn default_minimal_mode() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StoredConfig {
    pub base_url: String,
    pub selected_participant_ids: Vec<i64>,
    pub selection_initialized: bool,
    #[serde(default = "default_minimal_mode")]
    pub minimal_mode: bool,
    pub card_width: Option<f64>,
}

impl Default for StoredConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            selected_participant_ids: Vec::new(),
            selection_initialized: false,
            minimal_mode: true,
            card_width: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    #[default]
    Card,
    Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Snapshot {
    pub participant_id: i64,
    pub participant_name: String,
    pub source_sub2api_user_id: Option<i64>,
    pub quota_pool_id: Option<i64>,
    pub quota_pool_name: String,
    pub pool_contract_revision: Option<i64>,
    pub share_percent: Option<f64>,
    pub selected_cost: f64,
    pub delta_cost: Option<f64>,
    pub charged_delta_percent: f64,
    pub charged_cycle_percent: f64,
    pub charged_percent_lower: Option<f64>,
    pub charged_percent_upper: Option<f64>,
    pub remaining_share_percent: f64,
    pub current_balance_usd: Option<f64>,
    pub recommended_balance_usd: Option<f64>,
    pub recommended_balance_min_usd: Option<f64>,
    pub recommended_balance_max_usd: Option<f64>,
    pub deterministic_balance_min_usd: Option<f64>,
    pub deterministic_balance_max_usd: Option<f64>,
    pub balance_difference_usd: Option<f64>,
    pub is_overused: bool,
    pub overused_percent: f64,
    pub overused_percent_min: f64,
    pub overused_percent_max: f64,
    pub needs_manual_update: bool,
    pub recommendation_applied: bool,
    pub reason: String,
    pub allocation_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccountBreakdown {
    pub id: Option<i64>,
    pub account_id: i64,
    pub external_account_id: i64,
    pub account_name: String,
    pub account_enabled: bool,
    pub pool_id: i64,
    pub pool_name: String,
    pub contract_share_percent: f64,
    pub allocated: bool,
    pub latest_selected_cost: Option<f64>,
    pub last_checked_at: Option<String>,
    pub snapshot: Option<Snapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AggregateRecommendationSource {
    pub account_id: i64,
    pub external_account_id: i64,
    pub account_name: String,
    pub pool_id: i64,
    pub pool_name: String,
    pub pool_contract_revision: i64,
    pub contract_share_percent: f64,
    pub snapshot: Option<Snapshot>,
    pub net_position_usd: Option<f64>,
    pub net_position_min_usd: Option<f64>,
    pub net_position_max_usd: Option<f64>,
    pub contribution_usd: Option<f64>,
    pub contribution_min_usd: Option<f64>,
    pub contribution_max_usd: Option<f64>,
    pub estimated_capacity_usd: Option<f64>,
    pub expected_entitlement_usd: Option<f64>,
    pub consumed_entitlement_usd: Option<f64>,
    pub remaining_entitlement_usd: Option<f64>,
    pub entitlement_usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ParticipantPoolAllocation {
    pub pool_id: i64,
    pub pool_name: String,
    pub share_percent: f64,
    pub account_ids: Vec<i64>,
    pub account_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AggregateRecommendation {
    pub participant_id: i64,
    pub participant_name: String,
    pub pool_allocations: Vec<ParticipantPoolAllocation>,
    pub selected_cost: f64,
    pub charged_cycle_percent: f64,
    pub expected_entitlement_usd: Option<f64>,
    pub consumed_entitlement_usd: Option<f64>,
    pub remaining_entitlement_usd: Option<f64>,
    pub entitlement_usage_percent: Option<f64>,
    pub current_balance_usd: Option<f64>,
    pub recommended_balance_usd: Option<f64>,
    pub recommended_balance_min_usd: Option<f64>,
    pub recommended_balance_max_usd: Option<f64>,
    pub balance_difference_usd: Option<f64>,
    pub is_overused: bool,
    pub needs_manual_update: bool,
    pub recommendation_applied: bool,
    pub recommendation_complete: bool,
    pub account_count: i64,
    pub pool_count: i64,
    pub reason: String,
    pub allocation_model: String,
    pub sources: Vec<AggregateRecommendationSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ParticipantCardData {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub sub2api_user_id: i64,
    pub sub2api_username: String,
    pub sub2api_email: String,
    pub sub2api_identity: String,
    pub pool_allocations: Vec<ParticipantPoolAllocation>,
    pub is_owner: bool,
    pub enabled: bool,
    pub notes: String,
    pub latest_balance_usd: Option<f64>,
    pub last_checked_at: Option<String>,
    pub account_breakdowns: Vec<AccountBreakdown>,
    pub snapshot: Option<AggregateRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppSnapshot {
    pub configured: bool,
    pub base_url: String,
    pub selected_participant_ids: Vec<i64>,
    pub autostart_enabled: bool,
    pub minimal_mode: bool,
    pub card_width: Option<f64>,
    pub display_mode: DisplayMode,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshResult {
    pub participants: Vec<ParticipantCardData>,
    pub actionable_participant_ids: Vec<i64>,
    pub selected_participant_ids: Vec<i64>,
    pub refreshed_at_ms: u64,
}

pub struct AppState {
    pub config: Mutex<StoredConfig>,
    pub participants: Mutex<Vec<ParticipantCardData>>,
    pub display_mode: Mutex<DisplayMode>,
}

impl AppState {
    pub fn new(config: StoredConfig) -> Self {
        Self {
            config: Mutex::new(config),
            participants: Mutex::new(Vec::new()),
            display_mode: Mutex::new(DisplayMode::Card),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StoredConfig;

    #[test]
    fn minimal_mode_defaults_to_enabled() {
        let default = StoredConfig::default();
        assert!(default.minimal_mode);
        assert_eq!(default.card_width, None);
        let config: StoredConfig = serde_json::from_str("{}").expect("config should deserialize");
        assert!(config.minimal_mode);
        assert_eq!(config.card_width, None);
        let saved: StoredConfig =
            serde_json::from_str(r#"{"card_width":812}"#).expect("width should deserialize");
        assert_eq!(saved.card_width, Some(812.0));
    }
}
