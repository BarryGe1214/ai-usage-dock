import type {
  AccountView,
  AppConfig,
  Bootstrap,
  CodexCheck,
  PreferencesUpdate,
  UsageSnapshot,
} from "../types";

const now = Date.now();
const iso = (offsetMs: number) => new Date(now + offsetMs).toISOString();

const baseConfig: AppConfig = {
  language: "zh",
  theme: "system",
  uiMode: "floating",
  sidebarEdge: "right",
  refreshIntervalSecs: 120,
  alwaysOnTop: true,
  startAtLogin: true,
  floatingExpanded: false,
  funcloudBaseUrl: "https://api.funcloud.ai",
};

const accounts: AccountView[] = [
  {
    id: "codex-default",
    provider: "codex",
    label: "OpenAI Codex",
    enabled: true,
    hasSecret: true,
  },
  {
    id: "funcloud-work",
    provider: "funcloud",
    label: "工作 API",
    enabled: true,
    hasSecret: true,
  },
  {
    id: "funcloud-personal",
    provider: "funcloud",
    label: "个人 API",
    enabled: true,
    hasSecret: true,
  },
];

function dailySeries(base: number[]): UsageSnapshot["daily"] {
  return base.map((cost, index) => {
    const date = new Date(now - (6 - index) * 86_400_000);
    return {
      date: date.toISOString().slice(0, 10),
      reqCnt: 18 + index * 4,
      inputs: 120_000 + index * 21_000,
      outputs: 22_000 + index * 4_500,
      cacheReads: 62_000 + index * 8_000,
      cacheWrites: 3_200 + index * 400,
      cost,
    };
  });
}

const snapshots: UsageSnapshot[] = [
  {
    accountId: "codex-default",
    provider: "codex",
    label: "OpenAI Codex",
    status: "ok",
    fetchedAt: iso(-42_000),
    windows: [
      {
        id: "codex-primary",
        windowSec: 18_000,
        enabled: true,
        limit: 100,
        used: 27,
        remaining: 73,
        percent: 27,
        resetsAt: iso(2.4 * 3_600_000),
      },
      {
        id: "codex-secondary",
        windowSec: 604_800,
        enabled: true,
        limit: 100,
        used: 54,
        remaining: 46,
        percent: 54,
        resetsAt: iso(4.2 * 86_400_000),
      },
    ],
    daily: [],
    models: [],
    codex: {
      planType: "plus",
      limitName: "Codex",
      reachedType: null,
      credits: { balance: 12.5, unlimited: false },
    },
  },
  {
    accountId: "funcloud-work",
    provider: "funcloud",
    label: "工作 API",
    status: "ok",
    fetchedAt: iso(-38_000),
    windows: [
      {
        id: "funcloud-18000",
        windowSec: 18_000,
        enabled: true,
        limit: 20,
        used: 4.2,
        remaining: 15.8,
        percent: 21,
        resetsAt: iso(1.8 * 3_600_000),
      },
      {
        id: "funcloud-604800",
        windowSec: 604_800,
        enabled: true,
        limit: 50,
        used: 18.6,
        remaining: 31.4,
        percent: 37.2,
        resetsAt: iso(3.6 * 86_400_000),
      },
    ],
    summary: {
      todayCost: 2.34,
      rpm: 3.2,
      tpm: 1_250,
      totalReqCnt: 284,
      totalInputs: 2_840_000,
      totalOutputs: 412_000,
      totalTokens: 3_252_000,
      totalCacheWrites: 24_000,
      totalCacheReads: 980_000,
      totalCost: 18.62,
      avgLatencyMs: 820,
    },
    daily: dailySeries([1.12, 0.84, 2.05, 1.44, 3.2, 2.1, 2.34]),
    models: [
      {
        model: "gpt-5.1-codex",
        reqCnt: 122,
        inputs: 1_420_000,
        outputs: 210_000,
        totalTokens: 1_630_000,
        cost: 9.84,
      },
      {
        model: "gpt-5.1",
        reqCnt: 74,
        inputs: 820_000,
        outputs: 128_000,
        totalTokens: 948_000,
        cost: 5.2,
      },
      {
        model: "gpt-4.1-mini",
        reqCnt: 88,
        inputs: 600_000,
        outputs: 74_000,
        totalTokens: 674_000,
        cost: 3.58,
      },
    ],
  },
  {
    accountId: "funcloud-personal",
    provider: "funcloud",
    label: "个人 API",
    status: "ok",
    fetchedAt: iso(-36_000),
    windows: [
      {
        id: "funcloud-18000",
        windowSec: 18_000,
        enabled: true,
        limit: 10,
        used: 1.1,
        remaining: 8.9,
        percent: 11,
        resetsAt: iso(1.2 * 3_600_000),
      },
      {
        id: "funcloud-604800",
        windowSec: 604_800,
        enabled: true,
        limit: 30,
        used: 3.2,
        remaining: 26.8,
        percent: 10.7,
        resetsAt: iso(5.1 * 86_400_000),
      },
    ],
    summary: {
      todayCost: 0.42,
      rpm: 0.4,
      tpm: 180,
      totalReqCnt: 36,
      totalInputs: 240_000,
      totalOutputs: 42_000,
      totalTokens: 282_000,
      totalCacheWrites: 1_200,
      totalCacheReads: 68_000,
      totalCost: 3.2,
      avgLatencyMs: 640,
    },
    daily: dailySeries([0.1, 0.24, 0.08, 0.52, 0.2, 0.36, 0.42]),
    models: [
      {
        model: "gpt-5.1-mini",
        reqCnt: 22,
        inputs: 180_000,
        outputs: 32_000,
        totalTokens: 212_000,
        cost: 2.1,
      },
      {
        model: "gpt-4.1-mini",
        reqCnt: 14,
        inputs: 60_000,
        outputs: 10_000,
        totalTokens: 70_000,
        cost: 1.1,
      },
    ],
  },
];

let config = structuredClone(baseConfig);
let accountState = structuredClone(accounts);
let snapshotState = structuredClone(snapshots);

export function mockBootstrap(): Bootstrap {
  return {
    config: structuredClone(config),
    accounts: structuredClone(accountState),
    snapshots: structuredClone(snapshotState),
    version: "0.1.0-preview",
    platform: "browser",
    codexPath: "C:\\Users\\demo\\AppData\\Local\\OpenAI\\Codex\\bin\\codex.exe",
  };
}

export function mockRefreshAll(): UsageSnapshot[] {
  const stamp = new Date().toISOString();
  snapshotState = snapshotState.map((snapshot) => {
    if (snapshot.status !== "ok") return snapshot;
    return {
      ...snapshot,
      fetchedAt: stamp,
      windows: snapshot.windows.map((window, index) => {
        const delta = ((Date.now() / 60_000 + index) % 3) * 0.02;
        const used = Math.min(window.limit || 100, window.used + delta);
        const remaining = Math.max(0, (window.limit || 100) - used);
        return {
          ...window,
          used,
          remaining,
          percent: window.limit > 0 ? (used / window.limit) * 100 : window.percent + delta,
        };
      }),
    };
  });
  return structuredClone(snapshotState);
}

export function mockRefreshAccount(id: string): UsageSnapshot[] {
  mockRefreshAll();
  return structuredClone(snapshotState.filter((snapshot) => snapshot.accountId === id));
}

export function mockUpdatePreferences(preferences: PreferencesUpdate): AppConfig {
  config = { ...config, ...preferences };
  return structuredClone(config);
}

export function mockAddAccount(label: string): AccountView {
  const id = `funcloud-mock-${Date.now()}`;
  const account: AccountView = {
    id,
    provider: "funcloud",
    label: label || "新账号",
    enabled: true,
    hasSecret: true,
  };
  accountState = [...accountState, account];
  snapshotState = [
    ...snapshotState,
    {
      accountId: id,
      provider: "funcloud",
      label: account.label,
      status: "loading",
      fetchedAt: new Date().toISOString(),
      windows: [],
      daily: [],
      models: [],
    },
  ];
  return structuredClone(account);
}

export function mockUpdateAccount(id: string, label: string): AccountView {
  accountState = accountState.map((account) =>
    account.id === id ? { ...account, label } : account,
  );
  snapshotState = snapshotState.map((snapshot) =>
    snapshot.accountId === id ? { ...snapshot, label } : snapshot,
  );
  return structuredClone(accountState.find((account) => account.id === id)!);
}

export function mockRemoveAccount(id: string): void {
  accountState = accountState.filter((account) => account.id !== id);
  snapshotState = snapshotState.filter((snapshot) => snapshot.accountId !== id);
}

export function mockSetEnabled(id: string, enabled: boolean): AppConfig {
  accountState = accountState.map((account) =>
    account.id === id ? { ...account, enabled } : account,
  );
  snapshotState = snapshotState.map((snapshot) =>
    snapshot.accountId === id
      ? { ...snapshot, status: enabled ? "loading" : "disabled" }
      : snapshot,
  );
  return structuredClone(config);
}

export function mockCheckCodex(): CodexCheck {
  return {
    installed: true,
    path: "C:\\Users\\demo\\AppData\\Local\\OpenAI\\Codex\\bin\\codex.exe",
    loggedIn: true,
    planType: "plus",
    error: null,
  };
}
