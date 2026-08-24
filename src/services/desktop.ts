import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import {
  previewApplyRecommendation,
  previewRefreshResult,
  previewSnapshot,
} from "../preview";
import type { AppSnapshot, DisplayMode, RefreshResult } from "../types";

const previewMode = import.meta.env.DEV && !isTauri();

function previewCardData(): RefreshResult {
  const result = previewRefreshResult();
  if (!new URLSearchParams(window.location.search).has("system-user")) {
    return result;
  }
  const participants = result.participants.slice(0, 1);
  return {
    ...result,
    participants,
    actionable_participant_ids: [],
    selected_participant_ids: participants.map((participant) => participant.id),
  };
}

export async function getAppSnapshot(): Promise<AppSnapshot> {
  if (!previewMode) return invoke<AppSnapshot>("get_app_snapshot");
  const params = new URLSearchParams(window.location.search);
  if (params.has("settings")) {
    return {
      ...previewSnapshot,
      configured: false,
      base_url: "",
      display_mode: "settings",
    };
  }
  return params.has("full")
    ? { ...previewSnapshot, minimal_mode: false }
    : { ...previewSnapshot };
}

export async function refreshParticipants(): Promise<RefreshResult> {
  return previewMode
    ? previewCardData()
    : invoke<RefreshResult>("refresh_participants");
}

export async function applyParticipantRecommendation(
  participantId: number,
): Promise<RefreshResult> {
  return previewMode
    ? previewApplyRecommendation(participantId)
    : invoke<RefreshResult>("apply_participant_recommendation", {
        participantId,
      });
}

export async function saveConnection(
  baseUrl: string,
  token: string,
): Promise<RefreshResult> {
  return previewMode
    ? previewCardData()
    : invoke<RefreshResult>("save_connection", { baseUrl, token });
}

export async function fitWindowToContent(
  width: number,
  height: number,
): Promise<void> {
  if (!previewMode) await invoke("fit_window_to_content", { width, height });
}

export async function saveCardWidth(width: number): Promise<void> {
  if (previewMode) {
    previewSnapshot.card_width = width;
  } else {
    await invoke("save_card_width", { width });
  }
}

export async function startLeftResize(): Promise<void> {
  if (!previewMode) {
    await getCurrentWindow().startResizeDragging("West");
  }
}


export async function hideCard(): Promise<void> {
  if (!previewMode) await invoke("hide_card");
}

async function listenOrNoop<T>(
  event: string,
  callback: (payload: T) => void,
): Promise<UnlistenFn> {
  if (previewMode) return () => undefined;
  return listen<T>(event, ({ payload }) => callback(payload));
}

export function onSelectionChanged(
  callback: (ids: number[]) => void,
): Promise<UnlistenFn> {
  return listenOrNoop("selection-changed", callback);
}

export function onRefreshRequested(callback: () => void): Promise<UnlistenFn> {
  return listenOrNoop("refresh-requested", callback);
}

export function onDisplayModeChanged(
  callback: (mode: DisplayMode) => void,
): Promise<UnlistenFn> {
  return listenOrNoop("display-mode", callback);
}

export function onCardVisibilityChanged(
  callback: (visible: boolean) => void,
): Promise<UnlistenFn> {
  return listenOrNoop("card-visibility", callback);
}

export function onMinimalModeChanged(
  callback: (enabled: boolean) => void,
): Promise<UnlistenFn> {
  return listenOrNoop("minimal-mode-changed", callback);
}

export function onDesktopError(
  callback: (message: string) => void,
): Promise<UnlistenFn> {
  return listenOrNoop("desktop-error", callback);
}
