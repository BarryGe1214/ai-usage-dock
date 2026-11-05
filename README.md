# AI Usage Dock

一个面向 Windows 11 的 AI 额度桌面小组件，用一个窗口同时查看 **OpenAI Codex** 订阅额度和 **Funcloud** API Key 用量。支持悬浮窗、贴边侧栏、系统托盘面板三种形态，并预留 Linux / macOS 构建。

> 界面支持简体中文与 English，默认中文。

## 功能

- **OpenAI Codex 额度**：读取本机 Codex CLI 当前登录账号的 5 小时 / 7 天额度、重置时间、套餐和 Credits。
- **Funcloud 多账号**：同一个供应商可添加多个 API Key，分别显示 5 小时、每日、每周、每月或总额度窗口，以及今日费用、累计费用、Tokens、RPM / TPM、近 7 天费用和模型用量。
- **三种应用内显示形态**：
  - 悬浮窗：无边框、置顶、可拖动，紧凑 / 展开两种高度。
  - 贴边侧栏：吸附屏幕左侧或右侧，占满可用高度。
  - 任务栏面板：340px 宽的面板固定在任务栏上方、屏幕右下角，常驻显示，不因失焦隐藏。
- **Windows 11 系统小组件**：独立 MSIX 提供的原生 Widgets Board 小组件（小 / 中尺寸），可在 `Win+W` 小组件面板中固定，显示 Codex 剩余额度与 Funcloud 今日费用。
- **托盘常驻**：托盘图标显示聚合额度提示，右键菜单可刷新、切换形态、打开设置或退出。
- **退出方式**：标题栏电源按钮、设置页“退出应用”、托盘右键“退出”均可关闭程序；隐藏按钮只是收起窗口。
- **内置设置页**：点击齿轮后主窗口直接切换到设置视图，关闭后恢复原来的形态和位置。
- **应用内更新**：启动后自动检查新版本，主界面顶部提示；设置页可手动检查、下载、一键重启完成更新。
- **自动发布**：推送到 `main` 且版本号变化时，自动打 tag、构建 Windows 安装包并发布 Release。
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

从 [Releases](https://github.com/BarryGe1214/ai-usage-dock/releases) 下载最新版本：

- `AI.Usage.Dock_<版本>_x64-setup.exe`：NSIS 当前用户安装（推荐）
- `AI.Usage.Dock_<版本>_x64_en-US.msi`：MSI 安装包

安装后可以在应用内检查更新（设置 → 关于 → 软件更新），无需手动下载新版本。

当前构建未做代码签名，Windows 可能显示“未知发布者”。确认来源可信后选择“更多信息 → 仍要运行”。

### Windows 11 系统小组件（Widgets Board）

> 这是 Windows 11 自带的 `Win+W` 小组件面板里的原生卡片，不是应用的悬浮窗。它需要单独安装一个 MSIX 包，只有 Windows 11 22H2（build 22621）及以上支持。

Release 中会附带三个文件：

- `AIUsageDock.Widget.msix`：小组件 Provider 本体
- `AIUsageDock.Widget.cer`：自签名证书
- `install-widget.ps1`：安装脚本

安装步骤：

1. 先安装并运行主程序，让它至少完成一次额度刷新（主程序会把数据写入 `%LOCALAPPDATA%\com.barryge1214.aiusagedock\widget.json`）。
2. 把上面三个文件放在同一个目录。
3. 右键 `install-widget.ps1` → **使用 PowerShell 运行**；或在 PowerShell 中执行：

   ```powershell
   powershell -ExecutionPolicy Bypass -File .\install-widget.ps1
   ```

   脚本会把证书导入 `CurrentUser\TrustedPeople`，然后执行 `Add-AppxPackage`。
4. 按 `Win+W` 打开小组件面板 → 点击 **+ 添加小组件** → 选择 **AI Usage Dock** 并固定。

说明：

- 这是侧载（sideload）安装，必须信任自签名证书。若提示证书不受信任（`0x800B0109`），用管理员 PowerShell 再导入一次到机器级：

  ```powershell
  Import-Certificate -FilePath .\AIUsageDock.Widget.cer -CertStoreLocation Cert:\LocalMachine\TrustedPeople
  ```

- 小组件读取主程序导出的数据；主程序在后台刷新后，卡片会在 30 秒内更新，也可以点击卡片上的“刷新”。
- 如果 `Win+W` 的小组件库里暂时看不到它，先确认安装成功（`Get-AppxPackage BarryGe1214.AIUsageDock.Widget`），然后重启一次 `Widgets.exe` 或重启系统。
- 卸载：**设置 → 应用 → 已安装的应用 → AI Usage Dock Widget → 卸载**。
- 如果后续 Release 更换了签名证书，重新运行一次 `install-widget.ps1` 即可。

#### 配置固定的小组件签名证书（可选，推荐）

默认情况下 CI 每次发布会生成临时证书，升级小组件后需要重新信任新的 `.cer`。如果希望所有版本使用同一张证书：

1. 在本地 PowerShell 生成一张代码签名证书：

   ```powershell
   $cert = New-SelfSignedCertificate -Type Custom -Subject "CN=AI Usage Dock" `
     -KeyUsage DigitalSignature -KeySpec Signature -FriendlyName "AI Usage Dock Widget Signing" `
     -CertStoreLocation Cert:\CurrentUser\My -KeyExportPolicy Exportable -NotAfter (Get-Date).AddYears(10) `
     -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3")
   $password = "换成你自己的密码"
   Export-PfxCertificate -Cert $cert -FilePath .\AIUsageDock.Widget.pfx `
     -Password (ConvertTo-SecureString $password -Force -AsPlainText)
   [Convert]::ToBase64String([IO.File]::ReadAllBytes(".\AIUsageDock.Widget.pfx")) | Set-Clipboard
   ```

2. 在仓库 **Settings → Secrets and variables → Actions** 添加：

   | Secret | 内容 |
   | --- | --- |
   | `WIDGET_SIGNING_PFX_BASE64` | 上一步复制到剪贴板的 PFX Base64 |
   | `WIDGET_SIGNING_PFX_PASSWORD` | PFX 密码 |

未配置这两个 Secret 时构建不会失败，但每次发布都会使用新证书。

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

仓库已启用两套工作流：

- `.github/workflows/ci.yml`：push / PR 时构建前端（`tsc + vite`）并运行 Rust 单元测试。
- `.github/workflows/release.yml`：自动发布。

发布新版本只需要：

1. 修改 `src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`、`package.json` 中的版本号。
2. 提交并推送到 `main`。
3. 工作流读取 `tauri.conf.json` 的版本号，发现对应的 `vX.Y.Z` tag 不存在时自动创建 tag，在 Windows 上构建 NSIS / MSI 并发布 Release；版本号未变化时自动跳过。

也可以在 Actions 页面手动运行 `Release` 重新构建当前版本（默认生成 draft）。

### 应用内更新

应用启动后会在后台静默检查最新 Release，发现新版本时主界面顶部会出现更新提示。也可以随时在 **设置 → 关于 → 软件更新** 中手动检查：

- 点击“立即更新”下载并安装，界面会显示下载进度。
- 下载完成后点击“重启应用”切换到新版本。
- 更新包使用 Tauri 的 minisign 签名校验，防止被篡改。

#### 配置更新签名（仅需一次）

自动更新要求 Release 中包含签名后的安装包和 `latest.json`。在 GitHub 仓库的 **Settings → Secrets and variables → Actions** 中添加：

| Secret | 内容 |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | 私钥文件内容（`npx @tauri-apps/cli@2 signer generate -w ai-usage-dock.key` 生成，无密码时直接粘贴 `ai-usage-dock.key` 的全部内容） |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 生成密钥时设置的密码；没有密码则留空或不创建 |

公钥已经写入 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`。私钥只放在 GitHub Secrets 中，不要提交到仓库。

发布工作流在缺少 `TAURI_SIGNING_PRIVATE_KEY` 时会直接失败并给出提示，避免发布一个无法自动更新的版本。私钥一旦丢失，已经安装的版本将无法验证后续更新包，只能手动安装一次新版本，请妥善备份。

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
│  ├─ src/widget_feed.rs     # 导出 Windows 小组件数据（widget.json）
│  ├─ src/providers/         # Funcloud / Codex Provider
│  ├─ src/window.rs          # 三形态窗口定位
│  ├─ src/tray.rs            # 系统托盘与菜单
│  └─ src/credentials.rs     # 系统凭据库
├─ widget-provider/          # Windows 11 原生小组件（C# + MSIX）
│  ├─ UsageWidgetProvider.cs # IWidgetProvider 实现
│  ├─ WidgetTemplates.cs     # Adaptive Cards 模板
│  ├─ Package.appxmanifest   # MSIX 清单（COM Server + widgets appExtension）
│  ├─ build-widget.ps1       # 打包、自签名、生成安装脚本
│  └─ install-widget.ps1     # 终端用户安装脚本
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
- [ ] Win11 任务栏嵌入式形态验证（当前为任务栏上方的小组件，避免注入 explorer）

## English

AI Usage Dock is a Windows-first desktop widget for monitoring **OpenAI Codex** subscription limits and **Funcloud** API-key usage. It offers a floating widget, a docked sidebar, and a compact taskbar panel, plus a native Windows 11 Widgets Board provider (installed as a separate MSIX), with a bilingual UI, multiple Funcloud accounts, OS credential storage, and a cross-platform Tauri architecture.

- Codex data comes from the locally signed-in `codex app-server` (`account/rateLimits/read`), not from scraped ChatGPT cookies.
- Funcloud data comes from the same endpoints used by `https://funcloud.ai/key-usage`.
- Secrets are stored in Windows Credential Manager / Secret Service / Keychain.
- Windows installers are built by GitHub Actions (`.github/workflows/release.yml`); the same workflow builds the signed Windows 11 widget MSIX. CI runs the frontend build and Rust tests on every push. Linux and macOS builds are planned.
- The app checks GitHub Releases for signed updates on startup and can install them in place from **Settings → About → Software update**.

## Privacy

- No analytics, telemetry, or cloud sync.
- API keys never leave the local machine except in requests to the provider you configure.
- The app only sends requests to `https://api.funcloud.ai` (or your configured base URL) and to the local Codex CLI.

## License

[MIT](LICENSE) © 2026 BarryGe1214

本项目与 OpenAI、Funcloud 均无官方关联。
