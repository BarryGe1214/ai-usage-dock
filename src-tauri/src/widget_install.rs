use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

const PACKAGE_NAME: &str = "BarryGe1214.AIUsageDock.Widget";
const CERT_THUMBPRINT: &str = "C2A01C2B80A834302C5982151E0AA28F93A68911";
const RELEASES_URL: &str = "https://github.com/BarryGe1214/ai-usage-dock/releases/download";
const MIN_WINDOWS_BUILD: u32 = 22621;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const AUTO_INSTALL_DELAY: Duration = Duration::from_secs(12);
const FAILURE_MARKER: &str = "widget-install.json";
const RETRY_COOLDOWN_SECS: u64 = 6 * 60 * 60;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallAttempt {
    target_version: String,
    attempted_at: u64,
}

pub use crate::widget_install_status::WidgetStatus;

pub struct WidgetInstaller {
    status: Mutex<WidgetStatus>,
    busy: AtomicBool,
}

impl WidgetInstaller {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(WidgetStatus::new(target_version(), cfg!(windows))),
            busy: AtomicBool::new(false),
        }
    }

    fn status(&self) -> WidgetStatus {
        self.status
            .lock()
            .expect("widget status lock poisoned")
            .clone()
    }

    fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }

    fn update(&self, app: &AppHandle, apply: impl FnOnce(&mut WidgetStatus)) {
        let snapshot = {
            let mut status = self.status.lock().expect("widget status lock poisoned");
            apply(&mut *status);
            status.clone()
        };
        let _ = app.emit("widget://status", snapshot);
    }
}

fn target_version() -> String {
    format!("{}.0", env!("CARGO_PKG_VERSION"))
}

fn version_tuple(value: &str) -> [u64; 4] {
    let mut parts = [0u64; 4];
    for (index, part) in value.split('.').take(4).enumerate() {
        parts[index] = part.trim().parse().unwrap_or(0);
    }
    parts
}

#[tauri::command]
pub async fn widget_status(app: AppHandle) -> Result<WidgetStatus, String> {
    let installer = app.state::<WidgetInstaller>();
    let supported = blocking(is_supported).await?;
    if !supported {
        installer.update(&app, |status| {
            status.supported = false;
            status.phase = "unsupported".to_string();
        });
        return Ok(installer.status());
    }
    if !installer.is_busy() {
        let installed = blocking(query_installed_version).await?;
        installer.update(&app, |status| {
            status.supported = true;
            status.installed = installed.is_some();
            status.installed_version = installed;
        });
    }
    Ok(installer.status())
}

#[tauri::command]
pub async fn install_widget(app: AppHandle, force: bool) -> Result<WidgetStatus, String> {
    Ok(run_install(app, force).await)
}

/// Installs or upgrades the widget shortly after launch so an in-app update of
/// the desktop app also refreshes the MSIX that implements the native widget.
pub fn spawn_auto_install(app: AppHandle) {
    if !cfg!(windows) || cfg!(debug_assertions) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(AUTO_INSTALL_DELAY).await;
        let target = target_version();
        // Avoid downloading the 40+ MB package on every launch when this
        // machine cannot install it (policy, unsupported SKU, ...).
        if recent_failure(&app, &target) {
            return;
        }
        let status = run_install(app.clone(), false).await;
        if status.phase == "error" {
            record_failure(&app, &target);
        } else if status.installed {
            clear_failure(&app);
        }
    });
}

fn marker_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join(FAILURE_MARKER))
}

fn recent_failure(app: &AppHandle, target: &str) -> bool {
    let Some(path) = marker_path(app) else {
        return false;
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(attempt) = serde_json::from_str::<InstallAttempt>(&raw) else {
        return false;
    };
    attempt.target_version == target
        && unix_now().saturating_sub(attempt.attempted_at) < RETRY_COOLDOWN_SECS
}

fn record_failure(app: &AppHandle, target: &str) {
    let Some(path) = marker_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let attempt = InstallAttempt {
        target_version: target.to_string(),
        attempted_at: unix_now(),
    };
    if let Ok(raw) = serde_json::to_string(&attempt) {
        let _ = std::fs::write(path, raw);
    }
}

fn clear_failure(app: &AppHandle) {
    if let Some(path) = marker_path(app) {
        let _ = std::fs::remove_file(path);
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

async fn run_install(app: AppHandle, force: bool) -> WidgetStatus {
    let installer = app.state::<WidgetInstaller>();
    if installer.busy.swap(true, Ordering::SeqCst) {
        return installer.status();
    }

    let result = install_inner(&app, force).await;
    if let Err(error) = result {
        installer.update(&app, |status| {
            status.phase = "error".to_string();
            status.error = Some(error);
        });
    }

    let status = installer.status();
    if status.phase == "done" {
        clear_failure(&app);
    }
    installer.busy.store(false, Ordering::SeqCst);
    status
}

async fn install_inner(app: &AppHandle, force: bool) -> Result<(), String> {
    let installer = app.state::<WidgetInstaller>();

    let supported = blocking(is_supported).await?;
    if !supported {
        installer.update(app, |status| {
            status.supported = false;
            status.phase = "unsupported".to_string();
        });
        return Ok(());
    }

    installer.update(app, |status| {
        status.supported = true;
        status.phase = "checking".to_string();
        status.downloaded = 0;
        status.total = None;
        status.error = None;
    });

    let installed = blocking(query_installed_version).await?;
    installer.update(app, |status| {
        status.installed = installed.is_some();
        status.installed_version = installed.clone();
    });

    let target = target_version();
    if !force {
        if let Some(current) = &installed {
            if version_tuple(current) >= version_tuple(&target) {
                installer.update(app, |status| {
                    status.phase = "done".to_string();
                });
                return Ok(());
            }
        }
    }

    let work_dir = std::env::temp_dir().join(format!(
        "ai-usage-dock-widget-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or_default()
    ));
    tokio::fs::create_dir_all(&work_dir)
        .await
        .map_err(|error| format!("无法创建临时目录: {error}"))?;

    let outcome = install_package(app, &work_dir, force).await;
    let _ = tokio::fs::remove_dir_all(&work_dir).await;
    outcome?;

    let installed = blocking(query_installed_version).await?;
    let mut verification = Ok(());
    installer.update(app, |status| {
        verification = status.complete_install(installed);
    });
    verification
}

async fn install_package(
    app: &AppHandle,
    work_dir: &Path,
    allow_trust_prompt: bool,
) -> Result<(), String> {
    let installer = app.state::<WidgetInstaller>();
    let tag = format!("v{}", env!("CARGO_PKG_VERSION"));
    let base = format!("{RELEASES_URL}/{tag}");
    let msix_path = work_dir.join("AIUsageDock.Widget.msix");
    let cert_path = work_dir.join("AIUsageDock.Widget.cer");

    download_file(
        app,
        &format!("{base}/AIUsageDock.Widget.msix"),
        &msix_path,
        true,
    )
    .await?;

    installer.update(app, |status| {
        status.phase = "installing".to_string();
        status.downloaded = status.total.unwrap_or(status.downloaded);
    });

    download_file(
        app,
        &format!("{base}/AIUsageDock.Widget.cer"),
        &cert_path,
        false,
    )
    .await?;

    let script_path = write_install_script(work_dir).await?;
    blocking(move || run_powershell_file(&script_path, allow_trust_prompt)).await?;
    Ok(())
}

async fn download_file(
    app: &AppHandle,
    url: &str,
    destination: &Path,
    track_progress: bool,
) -> Result<(), String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let installer = app.state::<WidgetInstaller>();
    if track_progress {
        installer.update(app, WidgetStatus::begin_download);
    }
    let response = client()
        .get(url)
        .send()
        .await
        .map_err(|error| format!("无法下载小组件资源: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("下载小组件资源失败（HTTP {}）", response.status()));
    }

    let total = response.content_length();
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now() - Duration::from_secs(1);
    let mut file = tokio::fs::File::create(destination)
        .await
        .map_err(|error| format!("无法写入临时文件: {error}"))?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("下载中断: {error}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|error| format!("无法写入临时文件: {error}"))?;
        downloaded += chunk.len() as u64;

        if track_progress && last_emit.elapsed() >= Duration::from_millis(150) {
            last_emit = Instant::now();
            installer.update(app, |status| {
                status.downloaded = downloaded;
                status.total = total;
            });
        }
    }

    file.flush()
        .await
        .map_err(|error| format!("无法写入临时文件: {error}"))?;

    if track_progress {
        installer.update(app, |status| {
            status.downloaded = downloaded;
            status.total = total.or(Some(downloaded));
        });
    }
    Ok(())
}

async fn write_install_script(work_dir: &Path) -> Result<PathBuf, String> {
    let script_path = work_dir.join("install-widget.ps1");
    let script = include_str!("../../widget-provider/install-widget.ps1");
    if !script.contains(CERT_THUMBPRINT) {
        return Err("小组件安装脚本的证书指纹与主程序不一致".to_string());
    }
    // Windows PowerShell 5.1 needs a BOM to read non-ASCII script text.
    tokio::fs::write(&script_path, format!("\u{feff}{script}"))
        .await
        .map_err(|error| format!("无法写入安装脚本: {error}"))?;
    Ok(script_path)
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(concat!("ai-usage-dock/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(20))
            .timeout(Duration::from_secs(600))
            .build()
            .expect("failed to build widget download client")
    })
}

async fn blocking<T, F>(task: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| format!("后台任务失败: {error}"))?
}

fn is_supported() -> Result<bool, String> {
    if !cfg!(windows) {
        return Ok(false);
    }
    let build = run_powershell_command("[Environment]::OSVersion.Version.Build")?;
    Ok(build
        .trim()
        .parse::<u32>()
        .map(|value| value >= MIN_WINDOWS_BUILD)
        .unwrap_or(false))
}

fn query_installed_version() -> Result<Option<String>, String> {
    let script = format!(
        "$package = Get-AppxPackage -Name '{PACKAGE_NAME}' | Select-Object -First 1; \
         if ($package) {{ $package.Version.ToString() }}"
    );
    let output = run_powershell_command(&script)?;
    let version = output.trim();
    if version.is_empty() {
        Ok(None)
    } else {
        Ok(Some(version.to_string()))
    }
}

fn run_powershell_command(script: &str) -> Result<String, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        let script = format!(
            "[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false); \
             $OutputEncoding = [Console]::OutputEncoding; $ErrorActionPreference = 'Stop'; \
             try {{ {script} }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
        );
        let output = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|error| format!("无法启动 PowerShell: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                "PowerShell 执行失败".to_string()
            } else {
                stderr
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = script;
        Err("仅支持 Windows".to_string())
    }
}

fn run_powershell_file(script: &Path, allow_trust_prompt: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        let mut command = std::process::Command::new("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(script);
        if !allow_trust_prompt {
            command.arg("-NoTrustPrompt");
        }
        let output = command
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|error| format!("无法启动 PowerShell: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let message = if stderr.is_empty() { stdout } else { stderr };
            return Err(if message.is_empty() {
                format!(
                    "小组件安装失败，退出码 {}",
                    output.status.code().unwrap_or(-1)
                )
            } else if message.contains("WIDGET_TRUST_REQUIRED:") {
                "小组件证书尚未被本机信任（0x800B0109）。请点击安装，并在 Windows 提示中授权；如使用其他管理员账号授权，安装仍会继续在当前账号下完成。".to_string()
            } else {
                message
            });
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (script, allow_trust_prompt);
        Err("仅支持 Windows".to_string())
    }
}
