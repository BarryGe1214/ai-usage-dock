# AI Usage Dock

一个面向 Windows 11 的 AI 额度桌面小组件，用一个窗口同时查看 **OpenAI Codex** 订阅额度和 **Funcloud** API Key 用量。支持悬浮窗、贴边侧栏、系统托盘面板三种形态，并预留 Linux / macOS 构建。

> 界面支持简体中文与 English，默认中文。

## 功能

- **OpenAI Codex 额度**：读取本机 Codex CLI 当前登录账号的 5 小时 / 7 天额度、重置时间、套餐和 Credits。
- **Funcloud 多账号**：同一个供应商可添加多个 API Key，分别显示 5 小时、每日、每周、每月或总额度窗口，以及今日费用、累计费用、Tokens、RPM / TPM、近 7 天费用和模型用量。
- **三种显示形态**：
  - 悬浮窗：无边框、置顶、可拖动，紧凑 / 展开两种高度。
  - 贴边侧栏：吸附屏幕左侧或右侧，占满可用高度。
  - 托盘面板：点击 Windows 系统托盘图标弹出，失焦自动隐藏。
- **托盘常驻**：托盘图标显示聚合额度提示，右键菜单可刷新、切换形态、打开设置或退出。
- **双语与主题**：简体中文 / English，深色 / 浅色 / 跟随系统。
- **安全存储**：Funcloud API Key 保存在 Windows 凭据管理器（Linux 使用 Secret Service，macOS 使用 Keychain），不写入配置文件。
- **自动刷新**：默认每 2 分钟刷新一次，可在设置中调整；支持开机自启。

## 数据来源

| 供应商 | 来源 | 说明 |
| --- | --- | --- |
| OpenAI Codex | 本机 `codex app-server` 的 `account/rateLimits/read` | 与 Codex CLI 当前登录账号一致；不是 ChatGPT 网页 token 抓取 |
| Funcloud | `https://api.funcloud.ai/v1/model/key-usage/*` | `limit`、`summary`、`daily`、`models` 接口；仅使用 API Key 查询 |

Funcloud 的查询接口来自其公开用量页面 `https://funcloud.ai/key-usage` 所使用的同一套接口，实际 API 域名为 `api.funcloud.ai`。该接口不是正式文档化的 OpenAPI，若官方改版，Provider 可能需要同步调整。

Codex 的 `app-server` 协议由本机 Codex CLI 提供，不同 Codex 版本可能存在字段差异。

## 快速开始

### 运行要求

- Windows 10 / 11（首要支持 Windows 11 x64）
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)（Windows 11 通常已内置）
- 如果要读取 Codex 额度：安装并登录 [OpenAI Codex CLI](https://github.com/openai/codex)

### 安装

从 [Releases](https://github.com/BarryGe1214/ai-usage-dock/releases) 下载安装包（需要先启用 release workflow 并触发一次构建）：

- `AI Usage Dock_0.1.0_x64-setup.exe`：NSIS 当前用户安装
- `AI Usage Dock_0.1.0_x64_en-US.msi`：MSI 安装包

当前构建未做代码签名，Windows 可能显示“未知发布者”。确认来源可信后选择“更多信息 → 仍要运行”。

### 添加 Funcloud 账号

1. 打开托盘菜单 → 设置，或点击悬浮窗右上角齿轮。
2. 在“Funcloud 账号”中点击“添加账号”。
3. 填写账号名称并粘贴 API Key。
4. API Key 会写入系统凭据库；配置文件只保存账号名称和供应商。

### 读取 Codex 额度

AI Usage Dock 只会读取本机当前登录的 Codex 账号：

```bash
codex login
```

如果应用提示找不到 Codex，可在设置中点击“检测 Codex”查看路径，或设置环境变量 `CODEX_CLI_PATH` 指向 `codex.exe`。

## 本地开发

```bash
npm install
npm run tauri dev
```

仅预览前端（浏览器中使用示例数据）：

```bash
npm run dev
```

构建前端：

```bash
npm run build
```

构建 Windows 安装包：

```bash
npm run tauri build
```

产物位于 `src-tauri/target/release/bundle/`。

### GitHub Actions

`ci/github-actions/ci.yml.txt` 和 `ci/github-actions/release.yml.txt` 是可直接使用的 GitHub Actions 工作流内容：

- `ci.yml.txt`：push / PR 时构建前端并运行 Rust 测试。
- `release.yml.txt`：手动触发或推送 `v*` tag 时构建 Windows NSIS / MSI 安装包。

GitHub 对 `.github/workflows/` 下的文件要求额外的 `workflow` scope。创建本仓库时使用的自动化 token 只有 `repo` scope，因此工作流内容以 `.yml.txt` 形式提供。使用带 `workflow` 权限的 GitHub 账号把它们复制为正式路径并提交，即可启用：

```bash
mkdir -p .github/workflows
cp ci/github-actions/ci.yml.txt .github/workflows/ci.yml
cp ci/github-actions/release.yml.txt .github/workflows/release.yml
git add .github/workflows
git commit -m "ci: enable GitHub Actions"
git push
```

### Linux 依赖（预留）

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libxdo-dev
```

### macOS

需要 Xcode Command Line Tools。当前未提供签名公证的 macOS 发布包。

## 项目结构

```txt
ai-usage-dock/
├─ src/                      # React + TypeScript 前端
│  ├─ components/            # 悬浮窗 / 侧栏 / 托盘面板 / 设置页
│  ├─ lib/bridge.ts          # Tauri invoke 与事件封装（含浏览器 Mock）
│  ├─ i18n.ts                # 中英双语文案
│  └─ styles.css             # 主题与三形态布局
├─ src-tauri/
│  ├─ src/commands.rs        # Tauri 命令
│  ├─ src/state.rs           # 聚合刷新、缓存、定时任务
│  ├─ src/providers/         # Funcloud / Codex Provider
│  ├─ src/window.rs          # 三形态窗口定位
│  ├─ src/tray.rs            # 系统托盘与菜单
│  └─ src/credentials.rs     # 系统凭据库
├─ .github/workflows/        # CI 与 Windows Release 构建
└─ README.md
```

## 扩展新的供应商

Provider 只需要返回统一的 `UsageSnapshot`：

1. 在 `src-tauri/src/providers/` 新增模块，实现异步查询函数。
2. 在 `src-tauri/src/models.rs` 的 `ProviderKind` 增加类型。
3. 在 `src-tauri/src/state.rs` 的 `fetch_account` 分发到新 Provider。
4. 在前端 `AccountCard` 增加展示分支与 i18n 文案。

窗口形态、刷新调度、托盘、凭据存储都与供应商无关。

## 路线图

- [ ] 支持多个 Codex 账号（独立 `CODEX_HOME`，需要逐账号 `codex login`）
- [ ] OpenAI Platform 组织用量 / 费用接口（Admin API Key）
- [ ] Linux AppImage / deb 发布
- [ ] macOS 签名与公证
- [ ] Win11 任务栏嵌入式形态验证（当前使用系统托盘面板，避免注入 explorer）

## English

AI Usage Dock is a Windows-first desktop widget for monitoring **OpenAI Codex** subscription limits and **Funcloud** API-key usage. It offers a floating widget, a docked sidebar, and a system-tray panel, with a bilingual UI, multiple Funcloud accounts, OS credential storage, and a cross-platform Tauri architecture.

- Codex data comes from the locally signed-in `codex app-server` (`account/rateLimits/read`), not from scraped ChatGPT cookies.
- Funcloud data comes from the same endpoints used by `https://funcloud.ai/key-usage`.
- Secrets are stored in Windows Credential Manager / Secret Service / Keychain.
- Windows installers can be built by GitHub Actions once the workflow templates in `ci/github-actions/*.yml.txt` are copied to `.github/workflows/`. Linux and macOS builds are planned.

## Privacy

- No analytics, telemetry, or cloud sync.
- API keys never leave the local machine except in requests to the provider you configure.
- The app only sends requests to `https://api.funcloud.ai` (or your configured base URL) and to the local Codex CLI.

## License

[MIT](LICENSE) © 2026 BarryGe1214

本项目与 OpenAI、Funcloud 均无官方关联。
