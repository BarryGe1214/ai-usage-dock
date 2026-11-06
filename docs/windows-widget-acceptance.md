# Windows 原生小组件 P0 验收

本清单验证 Win+W Widgets Board 卡片。悬浮窗、侧栏和任务栏面板不能代替它。

## 测试版本与前提

本清单针对 v0.1.10 修复版。v0.1.7 可作为故障基线，但不包含本轮修复。完整验收须使用同一版本的主程序和小组件。

自动安装只在 Windows **release** 构建启用，debug 构建需点击设置页的安装按钮。主程序按自身版本从同名 GitHub Release 下载 MSIX/CER，因此完整修复版验收需要同一版本的主程序、MSIX 和固定证书。单独本地构建修改后的 v0.1.7 主程序仍会下载旧版 v0.1.7 Provider，不能验证 Provider 修复。

准备 Windows 11 22H2+ x64 测试机、可联网且可侧载的普通用户会话。**通过 GitHub MSIX 侧载的小组件还需在 Windows 设置中开启“开发人员模式”**；仅允许侧载可信应用、导入证书或完成 Add-AppxPackage 不等同于满足 Widgets 的发现条件。无需开启设备门户或设备发现。微软官方依据：[Widgets 示例的分发要求](https://github.com/microsoft/WindowsAppSDK-Samples/tree/main/Samples/Widgets#widgetprovider-app-distribution)。主程序至少配置一个可刷新的账号。升级场景另外准备已安装 v0.1.6 小组件的机器或虚拟机快照。保留原固定签名证书。

## 只读检查脚本

在仓库根目录运行 Windows PowerShell 5.1 或 PowerShell 7：

```powershell
powershell -NoProfile -File .\scripts\Test-WindowsWidget.ps1 -ExpectedVersion 0.1.10.0 -WatchSeconds 90 -ReportPath .\widget-acceptance.json
```

替换 ExpectedVersion 为被测 MSIX 的四段版本。脚本检查当前用户包注册、版本、Publisher、包状态、feed JSON，并等待主程序刷新产生新时间戳。看到提示后，在主程序点击刷新。报告仅包含这些检查结果和账号数量，不包含 API Key、账号名称、额度明细。

脚本不会安装/卸载、导入证书或修改系统策略。执行策略若拦截脚本，请按机器既有管理要求处理。脚本通过不能证明 Win+W 卡片渲染成功，仍需下面的人工观察。

## 场景与通过条件

| 场景 | 操作 | 通过条件 |
| --- | --- | --- |
| 全新安装 | 安装并启动主程序，约 12 秒后查看设置 | 未安装 → 下载百分比 → 安装中 → 目标版本；脚本的包检查通过；Win+W 可添加卡片 |
| 旧版升级 | v0.1.6 主程序应用内升级至被测版 | 原有固定卡片保留、MSIX 升至目标版本、小/中尺寸正常，无错误重复弹出 |
| 下载失败 | 在测试机临时断开网络后触发安装 | 设置显示失败；恢复网络后手动重试成功；不能把仍存在的旧版包报告为升级成功 |
| 侧载被策略阻止 | 使用已有受管测试机或专门测试快照 | 显示具体错误；自动失败后 6 小时内重启不重复下载；手动按钮仍允许重试。记录错误码，手动脚本也可能受同一策略限制 |
| 成功后重试冷却 | 自动失败后恢复环境并手动安装成功 | `%APPDATA%\com.barryge1214.aiusagedock\widget-install.json` 清除，不残留失败标记 |
| 数据更新 | 主程序刷新，保持 Win+W 卡片可见 | 脚本报告 feed 时间戳变化；30 秒左右卡片同步新值；点击卡片刷新可立即重读已有 feed（不会主动请求供应商） |
| 多卡片 | 固定两张卡片，主程序刷新后只点击第一张卡片刷新 | 第二张仍在下一周期更新；小/中尺寸均可更新 |
| 包占用 | Provider 运行期间升级 | 自动安装可完成；如失败，记录是否为 0x80073D02，不先结束进程掩盖问题 |
| 不支持的系统 | Windows 10 打开设置页 | 显示需 Windows 11 22H2+，安装按钮禁用，无小组件下载；运行脚本加 `-ExpectUnsupported` 仅检查 OS 分支 |

请回传：主程序版本、Windows build、各场景通过/失败、脚本 JSON、失败错误码及相关界面描述。截图请遮盖账号信息。无需提供 PAT、API Key、证书私钥或完整日志。

## 云端回归

```bash
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
dotnet run --project tests/WidgetProviderLogic/WidgetProviderLogic.csproj
```

Rust 安装状态单元测试随主项目测试执行。C# 测试直接编译实际 Provider 源码，替换 Windows 宿主边界，覆盖多卡片刷新及失败重试；它不能替代 COM 注册、文件共享语义、MSIX 安装和真实 Win+W 验收。

## 0x800B0109 证书信任修复

v0.1.7 仅向 CurrentUser\TrustedPeople 导入证书，截图中的 0x800B0109 表明部署仍未信任证书链。修复后的安装脚本核对固定指纹后，检查 LocalMachine\TrustedPeople；首次手动安装可弹出 UAC，仅提升导入证书这一步，包注册继续由原用户完成。自动安装使用 `-NoTrustPrompt`，缺少机器信任时返回可操作的错误，不突然弹窗。

额外验收：拒绝 UAC 后应显示错误且不继续安装；再次点击安装并授权后应成功；证书已受信任时再次安装不应弹 UAC。即便用其他管理员账号批准，也须在原用户运行 `Get-AppxPackage` 检查。不得把证书放入 Trusted Root，也不要关闭包签名验证。

本轮提供的 v0.1.7 修复验证包保留官方原始 MSIX，只更新安装脚本；可验证证书信任修复并恢复现有应用的小组件。它不包含正在开发的 Provider 和主程序二进制修复，完整回归仍需修复版构建。
