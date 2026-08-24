import type {
  AggregateRecommendation,
  AppSnapshot,
  Participant,
  RefreshResult,
  Snapshot,
} from "./types";

const rounded = (value: number) => Number(value.toFixed(2));

function accountSnapshot(
  participantId: number,
  participantName: string,
  poolId: number,
  poolName: string,
  sharePercent: number,
  selectedCost: number,
  chargedPercent: number,
): Snapshot {
  return {
    participant_id: participantId,
    participant_name: participantName,
    source_sub2api_user_id: 50 + participantId,
    quota_pool_id: poolId,
    quota_pool_name: poolName,
    pool_contract_revision: 1,
    share_percent: sharePercent,
    selected_cost: selectedCost,
    delta_cost: 12.4,
    charged_delta_percent: 3.2,
    charged_cycle_percent: chargedPercent,
    charged_percent_lower: chargedPercent - 1.1,
    charged_percent_upper: chargedPercent + 1.1,
    remaining_share_percent: Math.max(0, sharePercent - chargedPercent),
    current_balance_usd: null,
    recommended_balance_usd: null,
    recommended_balance_min_usd: null,
    recommended_balance_max_usd: null,
    deterministic_balance_min_usd: null,
    deterministic_balance_max_usd: null,
    balance_difference_usd: null,
    is_overused: chargedPercent > sharePercent,
    overused_percent: Math.max(0, chargedPercent - sharePercent),
    overused_percent_min: 0,
    overused_percent_max: 0,
    needs_manual_update: false,
    recommendation_applied: false,
    reason: "",
    allocation_model: "time_varying",
  };
}

function participant(
  id: number,
  name: string,
  email: string,
  sharePercent: number,
  chargedPercent: number,
  selectedCost: number,
  balance: number,
  recommendation: number,
  needsUpdate: boolean,
  isOwner: boolean,
  accountNames: string[],
): Participant {
  const poolId = 100 + id;
  const poolName = accountNames.length > 1 ? `${name}混池` : accountNames[0];
  const expectedPerAccount = recommendation / accountNames.length;
  const consumedPerAccount = selectedCost / accountNames.length;
  const account_breakdowns = accountNames.map((accountName, index) => {
    const accountId = id * 10 + index + 1;
    return {
      id: accountId,
      account_id: accountId,
      external_account_id: 1000 + accountId,
      account_name: accountName,
      account_enabled: true,
      pool_id: poolId,
      pool_name: poolName,
      contract_share_percent: sharePercent,
      allocated: true,
      latest_selected_cost: rounded(consumedPerAccount),
      last_checked_at: new Date(Date.now() - 92_000).toISOString(),
      snapshot: accountSnapshot(
        id,
        name,
        poolId,
        poolName,
        sharePercent,
        rounded(consumedPerAccount),
        rounded(chargedPercent / accountNames.length),
      ),
    };
  });
  const sources = account_breakdowns.map((breakdown) => {
    const remaining = Math.max(0, expectedPerAccount - consumedPerAccount);
    return {
      account_id: breakdown.account_id,
      external_account_id: breakdown.external_account_id,
      account_name: breakdown.account_name,
      pool_id: poolId,
      pool_name: poolName,
      pool_contract_revision: 1,
      contract_share_percent: sharePercent,
      snapshot: breakdown.snapshot,
      net_position_usd: rounded(remaining),
      net_position_min_usd: rounded(remaining * 0.92),
      net_position_max_usd: rounded(remaining * 1.08),
      contribution_usd: rounded(expectedPerAccount),
      contribution_min_usd: rounded(expectedPerAccount * 0.92),
      contribution_max_usd: rounded(expectedPerAccount * 1.08),
      estimated_capacity_usd: rounded(expectedPerAccount),
      expected_entitlement_usd: rounded(expectedPerAccount),
      consumed_entitlement_usd: rounded(consumedPerAccount),
      remaining_entitlement_usd: rounded(remaining),
      entitlement_usage_percent:
        expectedPerAccount > 0
          ? rounded((consumedPerAccount * 100) / expectedPerAccount)
          : 0,
    };
  });
  const pool_allocations = [
    {
      pool_id: poolId,
      pool_name: poolName,
      share_percent: sharePercent,
      account_ids: account_breakdowns.map((account) => account.account_id),
      account_count: account_breakdowns.length,
    },
  ];
  const remainingEntitlement = Math.max(0, recommendation - selectedCost);
  const snapshot: AggregateRecommendation = {
    participant_id: id,
    participant_name: name,
    pool_allocations,
    selected_cost: selectedCost,
    charged_cycle_percent: chargedPercent,
    expected_entitlement_usd: recommendation,
    consumed_entitlement_usd: selectedCost,
    remaining_entitlement_usd: rounded(remainingEntitlement),
    entitlement_usage_percent:
      recommendation > 0 ? rounded((selectedCost * 100) / recommendation) : 0,
    current_balance_usd: balance,
    recommended_balance_usd: recommendation,
    recommended_balance_min_usd: rounded(recommendation * 0.92),
    recommended_balance_max_usd: rounded(recommendation * 1.08),
    balance_difference_usd: recommendation - balance,
    is_overused: chargedPercent > sharePercent,
    needs_manual_update: needsUpdate,
    recommendation_applied: false,
    recommendation_complete: true,
    account_count: accountNames.length,
    pool_count: 1,
    reason: needsUpdate
      ? "全局余额与所有已分配池的剩余权益区间差异较大"
      : "全局余额处于所有已分配池的合计建议区间内，无需调整",
    allocation_model: "partitioned_pool_sum",
    sources,
  };
  return {
    id,
    name,
    email,
    sub2api_user_id: 50 + id,
    sub2api_username: email.split("@")[0],
    sub2api_email: email,
    sub2api_identity: email,
    pool_allocations,
    is_owner: isOwner,
    enabled: true,
    notes: "",
    latest_balance_usd: balance,
    last_checked_at: new Date(Date.now() - 92_000).toISOString(),
    account_breakdowns,
    snapshot,
  };
}

let participants = [
  participant(
    1,
    "林屿",
    "linyu@example.com",
    35,
    21.8,
    83.72,
    126.4,
    118,
    false,
    true,
    ["OpenAI Team A", "OpenAI Team B"],
  ),
  participant(
    2,
    "青禾",
    "qinghe@example.com",
    30,
    27.4,
    69.21,
    54.8,
    92,
    true,
    false,
    ["OpenAI Team A"],
  ),
  participant(
    3,
    "远川",
    "yuanchuan@example.com",
    35,
    18.2,
    71.06,
    101.2,
    104,
    false,
    false,
    ["OpenAI Team B"],
  ),
];

export const previewSnapshot: AppSnapshot = {
  configured: true,
  base_url: "https://sub2pool.example.com",
  selected_participant_ids: participants.map((item) => item.id),
  autostart_enabled: false,
  minimal_mode: true,
  card_width: null,
  display_mode: "card",
  visible: true,
};

export function previewRefreshResult(): RefreshResult {
  return {
    participants,
    actionable_participant_ids: participants
      .filter(
        (item) =>
          item.enabled &&
          item.snapshot?.needs_manual_update &&
          !item.snapshot.recommendation_applied,
      )
      .map((item) => item.id),
    selected_participant_ids: previewSnapshot.selected_participant_ids,
    refreshed_at_ms: Date.now(),
  };
}

export function previewApplyRecommendation(
  participantId: number,
): RefreshResult {
  const selected = participants.find((item) => item.id === participantId);
  const recommendation = selected?.snapshot?.recommended_balance_usd;
  if (!selected || recommendation == null || recommendation < 0) {
    throw new Error("该参与者暂无可应用建议");
  }
  participants = participants.map((item) =>
    item.id === participantId
      ? {
          ...item,
          latest_balance_usd: recommendation,
          snapshot: {
            ...item.snapshot!,
            current_balance_usd: recommendation,
            balance_difference_usd: 0,
            needs_manual_update: false,
            recommendation_applied: true,
          },
        }
      : item,
  );
  return previewRefreshResult();
}
