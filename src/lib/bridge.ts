import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  AccountView,
  AppConfig,
  Bootstrap,
  CodexCheck,
  LayoutEvent,
  PreferencesUpdate,
  UiMode,
  UsageSnapshot,
} from "../types";
import {
  mockAddAccount,
  mockBootstrap,
  mockCheckCodex,
  mockRefreshAccount,
  mockRefreshAll,
  mockRemoveAccount,
  mockSetEnabled,
  mockUpdateAccount,
  mockUpdatePreferences,
} from "./mock";

export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const noopUnlisten: UnlistenFn = () => {};
const mockDelay = () => new Promise((resolve) => setTimeout(resolve, 180));

export async function getBootstrap(): Promise<Bootstrap> {
  if (!isTauri) {
    await mockDelay();
    return mockBootstrap();
  }
  return invoke<Bootstrap>("bootstrap");
}

export async function updatePreferences(
  preferences: PreferencesUpdate,
): Promise<AppConfig> {
  if (!isTauri) {
    await mockDelay();
    return mockUpdatePreferences(preferences);
  }
  return invoke<AppConfig>("update_preferences", { preferences });
}

export async function addFuncloudAccount(
  label: string,
  apiKey: string,
): Promise<AccountView> {
  if (!isTauri) {
    await mockDelay();
    return mockAddAccount(label);
  }
  return invoke<AccountView>("add_funcloud_account", { label, apiKey });
}

export async function updateFuncloudAccount(
  id: string,
  label: string,
  apiKey: string | null,
): Promise<AccountView> {
  if (!isTauri) {
    await mockDelay();
    return mockUpdateAccount(id, label);
  }
  return invoke<AccountView>("update_funcloud_account", { id, label, apiKey });
}

export async function removeAccount(id: string): Promise<void> {
  if (!isTauri) {
    await mockDelay();
    mockRemoveAccount(id);
    return;
  }
  await invoke("remove_account", { id });
}

export async function setAccountEnabled(
  id: string,
  enabled: boolean,
): Promise<AppConfig> {
  if (!isTauri) {
    await mockDelay();
    return mockSetEnabled(id, enabled);
  }
  return invoke<AppConfig>("set_account_enabled", { id, enabled });
}

export async function refreshAll(): Promise<UsageSnapshot[]> {
  if (!isTauri) {
    await mockDelay();
    return mockRefreshAll();
  }
  return invoke<UsageSnapshot[]>("refresh_all");
}

export async function refreshAccount(id: string): Promise<UsageSnapshot[]> {
  if (!isTauri) {
    await mockDelay();
    return mockRefreshAccount(id);
  }
  return invoke<UsageSnapshot[]>("refresh_account", { id });
}

export async function setUiMode(mode: UiMode): Promise<AppConfig> {
  if (!isTauri) {
    return mockUpdatePreferences({ ...(await getBootstrap()).config, uiMode: mode });
  }
  return invoke<AppConfig>("set_ui_mode", { mode });
}

export async function setFloatingExpanded(expanded: boolean): Promise<AppConfig> {
  if (!isTauri) {
    const config = (await getBootstrap()).config;
    return mockUpdatePreferences({ ...config, floatingExpanded: expanded });
  }
  return invoke<AppConfig>("set_floating_expanded", { expanded });
}

export async function hideMain(): Promise<void> {
  if (!isTauri) return;
  await invoke("hide_main");
}

export async function openSettings(): Promise<void> {
  if (!isTauri) {
    window.location.assign(`${window.location.pathname}?view=settings`);
    return;
  }
  await invoke("open_settings");
}

export async function closeSettings(): Promise<void> {
  if (!isTauri) {
    window.location.assign(window.location.pathname);
    return;
  }
  await invoke("close_settings");
}

export async function quitApp(): Promise<void> {
  if (!isTauri) return;
  await invoke("quit_app");
}

export async function checkCodex(): Promise<CodexCheck> {
  if (!isTauri) {
    await mockDelay();
    return mockCheckCodex();
  }
  return invoke<CodexCheck>("check_codex");
}

export async function openUrl(url: string): Promise<void> {
  if (!isTauri) {
    window.open(url, "_blank", "noopener,noreferrer");
    return;
  }
  await invoke("open_url", { url });
}

export function onUsageUpdated(
  handler: (snapshots: UsageSnapshot[]) => void,
): Promise<UnlistenFn> {
  if (!isTauri) return Promise.resolve(noopUnlisten);
  return listen<UsageSnapshot[]>("usage://updated", (event) => handler(event.payload));
}

export function onConfigUpdated(
  handler: (config: AppConfig) => void,
): Promise<UnlistenFn> {
  if (!isTauri) return Promise.resolve(noopUnlisten);
  return listen<AppConfig>("config://updated", (event) => handler(event.payload));
}

export function onAccountsUpdated(
  handler: (accounts: AccountView[]) => void,
): Promise<UnlistenFn> {
  if (!isTauri) return Promise.resolve(noopUnlisten);
  return listen<AccountView[]>("accounts://updated", (event) => handler(event.payload));
}

export function onLayoutUpdated(
  handler: (layout: LayoutEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauri) return Promise.resolve(noopUnlisten);
  return listen<LayoutEvent>("ui://layout", (event) => handler(event.payload));
}
