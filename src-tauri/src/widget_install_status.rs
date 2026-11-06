use serde::Serialize;

/// Status of the Windows 11 widget package, mirrored to the settings window.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetStatus {
    pub supported: bool,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub target_version: String,
    pub phase: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub error: Option<String>,
}

impl WidgetStatus {
    pub fn new(target_version: String, supported: bool) -> Self {
        Self {
            supported,
            installed: false,
            installed_version: None,
            target_version,
            phase: if supported { "idle" } else { "unsupported" }.to_string(),
            downloaded: 0,
            total: None,
            error: None,
        }
    }

    pub fn begin_download(&mut self) {
        self.phase = "downloading".to_string();
        self.downloaded = 0;
        self.total = None;
        self.error = None;
    }

    pub fn complete_install(&mut self, installed: Option<String>) -> Result<(), String> {
        self.installed = installed.is_some();
        self.installed_version = installed;
        let error = match self.installed_version.as_deref() {
            None => Some("小组件安装后未能检测到已注册的包".to_string()),
            Some(version) if version != self.target_version => Some(format!(
                "小组件安装后版本不匹配：预期 {}，检测到 {version}",
                self.target_version
            )),
            Some(_) => None,
        };
        if let Some(error) = error {
            self.phase = "error".to_string();
            self.error = Some(error.clone());
            return Err(error);
        }
        self.phase = "done".to_string();
        self.error = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_starts_before_any_bytes_arrive_and_clears_retry_state() {
        let mut status = WidgetStatus::new("0.1.7.0".into(), true);
        status.phase = "error".into();
        status.downloaded = 100;
        status.total = Some(200);
        status.error = Some("interrupted".into());
        status.begin_download();
        assert_eq!(status.phase, "downloading");
        assert_eq!(status.downloaded, 0);
        assert_eq!(status.total, None);
        assert_eq!(status.error, None);
    }

    #[test]
    fn exact_target_is_reported_as_success() {
        let mut status = WidgetStatus::new("0.1.7.0".into(), true);
        status.complete_install(Some("0.1.7.0".into())).unwrap();
        assert!(status.installed);
        assert_eq!(status.phase, "done");
        assert_eq!(status.installed_version.as_deref(), Some("0.1.7.0"));
    }

    #[test]
    fn old_or_unexpected_package_is_not_a_successful_install() {
        for version in ["0.1.6.0", "0.1.8.0", "invalid"] {
            let mut status = WidgetStatus::new("0.1.7.0".into(), true);
            assert!(status.complete_install(Some(version.into())).is_err());
            assert!(status.installed); // Retain the actual package for diagnostics.
            assert_eq!(status.installed_version.as_deref(), Some(version));
            assert_eq!(status.phase, "error");
            assert!(status.error.is_some());
        }
    }

    #[test]
    fn missing_package_is_not_reported_as_done() {
        let mut status = WidgetStatus::new("0.1.7.0".into(), true);
        assert!(status.complete_install(None).is_err());
        assert!(!status.installed);
        assert_eq!(status.phase, "error");
    }

    #[test]
    fn non_windows_status_is_unsupported() {
        let status = WidgetStatus::new("0.1.7.0".into(), false);
        assert!(!status.supported);
        assert_eq!(status.phase, "unsupported");
    }
}
