<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";

import ParticipantCard from "@/components/ParticipantCard.vue";
import SetupView from "@/components/SetupView.vue";
import {
  applyParticipantRecommendation,
  fitWindowToContent,
  getAppSnapshot,
  closeCard as closeCardWindow,
  onCardVisibilityChanged,
  onDesktopError,
  onDisplayModeChanged,
  onMinimalModeChanged,
  onRefreshRequested,
  onSelectionChanged,
  refreshParticipants,
  saveConnection,
  saveCardWidth,
  startLeftResize,
} from "@/services/desktop";
import type { DisplayMode, Participant, RefreshResult } from "@/types";

const configured = ref(false);
const baseUrl = ref("");
const participants = ref<Participant[]>([]);
const actionableParticipantIds = ref<number[]>([]);
const selectedIds = ref<number[]>([]);
const mode = ref<DisplayMode>("card");
const minimalMode = ref(true);
const cardVisible = ref(false);
const loading = ref(false);
const initializing = ref(true);
const saving = ref(false);
const error = ref("");
const actionError = ref("");
const applyingParticipantId = ref<number | null>(null);
const currentIndex = ref(0);
const direction = ref<"next" | "previous">("next");
const unlisteners: UnlistenFn[] = [];
let refreshTimer: ReturnType<typeof setInterval> | null = null;
let contentObserver: ResizeObserver | null = null;
let resizeSettleTimer: ReturnType<typeof setTimeout> | null = null;
let leftResizeActive = false;
let windowFitTimer: ReturnType<typeof setTimeout> | null = null;
const contentRoot = ref<HTMLElement | null>(null);
const minimalWindowWidth = 640;
const fullWindowWidth = 752;
const cacheMaxAgeMs = 60_000;
let cardWindowWidth = minimalWindowWidth;
let customCardWindowWidth: number | null = null;
let lastRefreshedAtMs = 0;

const selectedParticipants = computed(() =>
  participants.value.filter((participant) =>
    selectedIds.value.includes(participant.id),
  ),
);
const currentParticipant = computed(
  () => selectedParticipants.value[currentIndex.value] ?? null,
);

watch(selectedParticipants, (items) => {
  if (!items.length) {
    currentIndex.value = 0;
  } else if (currentIndex.value >= items.length) {
    currentIndex.value = items.length - 1;
  }
});

function applyResult(result: RefreshResult) {
  const currentId = currentParticipant.value?.id;
  participants.value = result.participants;
  actionableParticipantIds.value = result.actionable_participant_ids;
  selectedIds.value = result.selected_participant_ids;
  lastRefreshedAtMs = result.refreshed_at_ms;
  if (currentId != null) {
    const nextIndex = selectedParticipants.value.findIndex(
      (participant) => participant.id === currentId,
    );
    currentIndex.value = nextIndex >= 0 ? nextIndex : 0;
  }
}
function cacheIsStale() {
  return (
    lastRefreshedAtMs === 0 ||
    Date.now() - lastRefreshedAtMs >= cacheMaxAgeMs
  );
}


async function refresh() {
  if (!configured.value || loading.value) return;
  loading.value = true;
  error.value = "";
  actionError.value = "";
  try {
    applyResult(await refreshParticipants());
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    loading.value = false;
  }
}

async function applySuggestion(participant: Participant) {
  const recommendation = participant.snapshot?.recommended_balance_usd;
  if (
    !actionableParticipantIds.value.includes(participant.id) ||
    recommendation == null ||
    recommendation < 0 ||
    participant.snapshot?.recommendation_applied ||
    applyingParticipantId.value != null
  ) {
    return;
  }
  applyingParticipantId.value = participant.id;
  actionError.value = "";
  error.value = "";
  try {
    applyResult(await applyParticipantRecommendation(participant.id));
  } catch (cause) {
    const message = cause instanceof Error ? cause.message : String(cause);
    actionError.value = message;
    error.value = message;
  } finally {
    applyingParticipantId.value = null;
  }
}

async function save(baseUrlValue: string, token: string) {
  saving.value = true;
  error.value = "";
  actionError.value = "";
  try {
    applyResult(await saveConnection(baseUrlValue, token));
    configured.value = true;
    baseUrl.value = baseUrlValue.trim().replace(/\/+$/, "");
    mode.value = "card";
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    saving.value = false;
  }
}

function move(delta: -1 | 1) {
  const count = selectedParticipants.value.length;
  if (applyingParticipantId.value != null) return;
  if (count <= 1) return;
  direction.value = delta > 0 ? "next" : "previous";
  currentIndex.value = (currentIndex.value + delta + count) % count;
}

function scheduleWindowFit() {
  if (initializing.value) return;
  if (leftResizeActive) return;
  if (windowFitTimer) clearTimeout(windowFitTimer);
  windowFitTimer = setTimeout(() => {
    const content = contentRoot.value;
    if (!content) return;
    const height = Math.ceil(content.scrollHeight);
    if (height <= 0) return;
    const width =
      mode.value === "card" ? cardWindowWidth : fullWindowWidth;
    void fitWindowToContent(width, height).catch((cause) => {
      error.value = cause instanceof Error ? cause.message : String(cause);
    });
  }, 220);
}

function scheduleResizeSettle() {
  if (resizeSettleTimer) clearTimeout(resizeSettleTimer);
  resizeSettleTimer = setTimeout(() => {
    const settledWidth = Math.round(window.innerWidth);
    leftResizeActive = false;
    cardWindowWidth = settledWidth;
    customCardWindowWidth = settledWidth;
    void saveCardWidth(settledWidth).catch((cause) => {
      error.value = cause instanceof Error ? cause.message : String(cause);
    });
    scheduleWindowFit();
  }, 260);
}

function handleWindowResize() {
  if (mode.value !== "card" || initializing.value) return;
  const width = Math.round(window.innerWidth);
  if (width <= 0) return;
  cardWindowWidth = width;
  if (leftResizeActive) {
    scheduleResizeSettle();
  } else {
    scheduleWindowFit();
  }
}

async function beginLeftResize(event: PointerEvent) {
  if (event.button !== 0 || mode.value !== "card") return;
  event.preventDefault();
  cardWindowWidth = Math.round(window.innerWidth);
  leftResizeActive = true;
  try {
    await startLeftResize();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    scheduleResizeSettle();
  }
}

async function closeCard() {
  await closeCardWindow();
}

function handleKeydown(event: KeyboardEvent) {
  if (mode.value !== "card") return;
  if (event.key === "ArrowLeft") move(-1);
  if (event.key === "ArrowRight") move(1);
  if (event.key === "Escape") void closeCard();
}

onMounted(async () => {
  window.addEventListener("resize", handleWindowResize);
  window.addEventListener("keydown", handleKeydown);
  contentObserver = new ResizeObserver(scheduleWindowFit);
  if (contentRoot.value) contentObserver.observe(contentRoot.value);
  unlisteners.push(
    await onMinimalModeChanged((value) => {
      cardWindowWidth =
        customCardWindowWidth ??
        (value ? minimalWindowWidth : fullWindowWidth);
      minimalMode.value = value;
      void nextTick(scheduleWindowFit);
    }),
    await onSelectionChanged((ids) => {
      const currentId = currentParticipant.value?.id;
      selectedIds.value = ids;
      const nextIndex = selectedParticipants.value.findIndex(
        (participant) => participant.id === currentId,
      );
      currentIndex.value = nextIndex >= 0 ? nextIndex : 0;
      void nextTick(scheduleWindowFit);
    }),
    await onRefreshRequested(() => void refresh()),
    await onDisplayModeChanged((value) => {
      mode.value = value;
      void nextTick(scheduleWindowFit);
    }),
    await onCardVisibilityChanged((value) => {
      cardVisible.value = value;
      if (value) {
        void nextTick(scheduleWindowFit);
        if (mode.value === "card" && cacheIsStale()) void refresh();
      }
    }),
    await onDesktopError((message) => {
      error.value = message;
    }),
  );

  try {
    const snapshot = await getAppSnapshot();
    configured.value = snapshot.configured;
    baseUrl.value = snapshot.base_url;
    selectedIds.value = snapshot.selected_participant_ids;
    customCardWindowWidth = snapshot.card_width;
    cardWindowWidth =
      customCardWindowWidth ??
      (snapshot.minimal_mode ? minimalWindowWidth : fullWindowWidth);
    minimalMode.value = snapshot.minimal_mode;
    mode.value = snapshot.configured ? snapshot.display_mode : "settings";
    cardVisible.value = snapshot.visible;
    if (snapshot.configured) {
      if (snapshot.cached_refresh) {
        applyResult(snapshot.cached_refresh);
        if (cacheIsStale()) {
          void refresh();
        }
      } else {
        await refresh();
      }
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
    mode.value = "settings";
  } finally {
    initializing.value = false;
  }
  await nextTick();
  scheduleWindowFit();

  refreshTimer = setInterval(() => {
    if (cardVisible.value && mode.value === "card") void refresh();
  }, cacheMaxAgeMs);
});

onUnmounted(() => {
  window.removeEventListener("resize", handleWindowResize);
  window.removeEventListener("keydown", handleKeydown);
  for (const unlisten of unlisteners) unlisten();
  if (refreshTimer) clearInterval(refreshTimer);
  contentObserver?.disconnect();
  if (windowFitTimer) clearTimeout(windowFitTimer);
  if (resizeSettleTimer) clearTimeout(resizeSettleTimer);
});
</script>

<template>
    <div
      v-if="mode === 'card' && configured"
      class="tray-resize-edge"
      role="separator"
      aria-orientation="vertical"
      aria-label="拖动调整卡片宽度"
      @pointerdown.stop="beginLeftResize"
      @dblclick.stop
    ></div>
  <main class="tray-scroll" @dblclick="closeCard">
    <div ref="contentRoot" class="tray-content">
    <SetupView
      v-if="mode === 'settings' || !configured"
      :base-url="baseUrl"
      :configured="configured"
      :busy="saving"
      :error="error"
      @save="save"
      @cancel="mode = 'card'"
    />

    <div v-else-if="initializing || (loading && !participants.length)" class="p-3">
      <section class="card bg-base-200 shadow-xs">
        <div class="card-body items-center py-16">
          <span class="loading loading-spinner loading-lg"></span>
        </div>
      </section>
    </div>

    <div v-else-if="error && !participants.length" class="p-3" @dblclick.stop>
      <section class="card bg-base-200 shadow-xs">
        <div class="card-body items-center py-12 text-center">
          <h2 class="card-title">无法读取参与者</h2>
          <p class="max-w-md text-sm opacity-70">{{ error }}</p>
          <div class="card-actions mt-3">
            <button class="btn" type="button" @click="mode = 'settings'">
              检查连接
            </button>
            <button class="btn btn-primary" type="button" @click="refresh">
              重试
            </button>
          </div>
        </div>
      </section>
    </div>

    <div v-else-if="!selectedParticipants.length" class="p-3">
      <section class="card bg-base-200 shadow-xs">
        <div class="card-body items-center py-12 text-center">
          <h2 class="card-title">未选择参与者</h2>
          <p class="text-sm opacity-60">
            右键单击任务栏托盘图标，然后勾选需要展示的人。
          </p>
        </div>
      </section>
    </div>

    <div v-else class="tray-carousel">
      <div v-if="loading" class="tray-loading badge badge-neutral gap-2 shadow-sm">
        <span class="loading loading-spinner loading-xs"></span>
        刷新中
      </div>
      <div
        v-else-if="error"
        class="tray-loading badge badge-warning shadow-sm"
        :title="actionError || error"
      >
        {{ actionError ? "应用失败" : "刷新失败" }}
      </div>

      <div
        v-if="selectedParticipants.length > 1"
        class="tray-switch-zone tray-switch-zone-left"
      >
        <button
          class="tray-switch btn btn-square"
          type="button"
          aria-label="上一个参与者"
          :disabled="applyingParticipantId != null"
          @click="move(-1)"
          @dblclick.stop
        >
          ‹
        </button>
      </div>

      <Transition :name="`slide-${direction}`" mode="out-in">
        <div
          v-if="currentParticipant"
          :key="currentParticipant.id"
          class="tray-card"
        >
          <ParticipantCard
            :participant="currentParticipant"
            :editable="false"
            :minimal="minimalMode"
            :actionable-recommendation="
              actionableParticipantIds.includes(currentParticipant.id)
            "
            :applying-recommendation="
              applyingParticipantId === currentParticipant.id
            "
            @apply-recommendation="applySuggestion"
          />
        </div>
      </Transition>

      <div
        v-if="selectedParticipants.length > 1"
        class="tray-switch-zone tray-switch-zone-right"
      >
        <button
          class="tray-switch btn btn-square"
          type="button"
          aria-label="下一个参与者"
          :disabled="applyingParticipantId != null"
          @click="move(1)"
          @dblclick.stop
        >
          ›
        </button>
      </div>
    </div>
    </div>
  </main>
</template>
