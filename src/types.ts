export interface Snapshot {
  participant_id: number;
  participant_name: string;
  source_sub2api_user_id?: number;
  quota_pool_id?: number | null;
  quota_pool_name?: string;
  pool_contract_revision?: number | null;
  share_percent?: number;
  selected_cost: number;
  delta_cost: number | null;
  charged_delta_percent: number;
  charged_cycle_percent: number;
  charged_percent_lower: number | null;
  charged_percent_upper: number | null;
  remaining_share_percent: number;
  current_balance_usd: number | null;
  recommended_balance_usd: number | null;
  recommended_balance_min_usd: number | null;
  recommended_balance_max_usd: number | null;
  deterministic_balance_min_usd: number | null;
  deterministic_balance_max_usd: number | null;
  balance_difference_usd: number | null;
  is_overused: boolean;
  overused_percent: number;
  overused_percent_min: number;
  overused_percent_max: number;
  needs_manual_update: boolean;
  recommendation_applied: boolean;
  reason: string;
  allocation_model: "time_varying" | "constant_average";
}

export interface AccountBreakdown {
  id: number | null;
  account_id: number;
  external_account_id: number;
  account_name: string;
  account_enabled: boolean;
  pool_id: number;
  pool_name: string;
  contract_share_percent: number;
  allocated: boolean;
  latest_selected_cost: number | null;
  last_checked_at: string | null;
  snapshot: Snapshot | null;
}

export interface AggregateRecommendationSource {
  account_id: number;
  external_account_id: number;
  account_name: string;
  pool_id: number;
  pool_name: string;
  pool_contract_revision: number;
  contract_share_percent: number;
  snapshot: Snapshot | null;
  net_position_usd: number | null;
  net_position_min_usd: number | null;
  net_position_max_usd: number | null;
  contribution_usd: number | null;
  contribution_min_usd: number | null;
  contribution_max_usd: number | null;
  estimated_capacity_usd: number | null;
  expected_entitlement_usd: number | null;
  consumed_entitlement_usd: number | null;
  remaining_entitlement_usd: number | null;
  entitlement_usage_percent: number | null;
}

export interface ParticipantPoolAllocation {
  pool_id: number;
  pool_name: string;
  share_percent: number;
  account_ids?: number[];
  account_count?: number;
}

export interface AggregateRecommendation {
  participant_id: number;
  participant_name: string;
  pool_allocations: ParticipantPoolAllocation[];
  selected_cost: number;
  charged_cycle_percent: number;
  expected_entitlement_usd: number | null;
  consumed_entitlement_usd: number | null;
  remaining_entitlement_usd: number | null;
  entitlement_usage_percent: number | null;
  current_balance_usd: number | null;
  recommended_balance_usd: number | null;
  recommended_balance_min_usd: number | null;
  recommended_balance_max_usd: number | null;
  balance_difference_usd: number | null;
  is_overused: boolean;
  needs_manual_update: boolean;
  recommendation_applied: boolean;
  recommendation_complete: boolean;
  account_count: number;
  pool_count: number;
  reason: string;
  allocation_model: "partitioned_pool_sum";
  sources: AggregateRecommendationSource[];
}

export interface Participant {
  id: number;
  name: string;
  email: string;
  sub2api_user_id: number;
  sub2api_username: string;
  sub2api_email: string;
  sub2api_identity: string;
  pool_allocations: ParticipantPoolAllocation[];
  is_owner: boolean;
  enabled: boolean;
  notes: string;
  latest_balance_usd: number | null;
  last_checked_at: string | null;
  account_breakdowns: AccountBreakdown[];
  snapshot: AggregateRecommendation | null;
}

export type DisplayMode = "card" | "settings";

export interface AppSnapshot {
  configured: boolean;
  base_url: string;
  selected_participant_ids: number[];
  autostart_enabled: boolean;
  minimal_mode: boolean;
  card_width: number | null;
  display_mode: DisplayMode;
  visible: boolean;
  cached_refresh: RefreshResult | null;
}

export interface RefreshResult {
  participants: Participant[];
  actionable_participant_ids: number[];
  selected_participant_ids: number[];
  refreshed_at_ms: number;
}
