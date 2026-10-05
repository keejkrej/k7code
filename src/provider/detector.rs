use std::collections::HashMap;
use std::process::Command;

use crate::model::ProviderKind;

#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub kind: ProviderKind,
    pub is_available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

pub struct ProviderDetector;

impl ProviderDetector {
    pub fn detect_all() -> HashMap<ProviderKind, ProviderStatus> {
        let providers = [
            ProviderKind::Claude,
            ProviderKind::Codex,
            ProviderKind::Cursor,
            ProviderKind::Grok,
            ProviderKind::OpenCode,
            ProviderKind::Antigravity,
            ProviderKind::Ollama,
        ];

        let mut results = HashMap::new();
        for kind in providers {
            results.insert(kind, Self::detect(kind));
        }
        results
    }

    pub fn detect(kind: ProviderKind) -> ProviderStatus {
        let bin_name = kind.default_binary_name();

        let output = if cfg!(target_os = "windows") {
            Command::new("where.exe").arg(bin_name).output()
        } else {
            Command::new("which").arg(bin_name).output()
        };

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let first_path = stdout.lines().next().unwrap_or("").trim().to_string();

                // Check version
                let ver_out = Command::new(&first_path).arg("--version").output();
                let version = ver_out.ok().and_then(|vo| {
                    if vo.status.success() {
                        Some(String::from_utf8_lossy(&vo.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

                ProviderStatus {
                    kind,
                    is_available: true,
                    path: Some(first_path),
                    version,
                }
            }
            _ => ProviderStatus {
                kind,
                is_available: false,
                path: None,
                version: None,
            },
        }
    }
}
