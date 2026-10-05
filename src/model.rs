use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub git_branch: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            path,
            git_branch: None,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeMode {
    #[serde(rename = "approval-required", alias = "read-only")]
    Supervised,
    #[serde(rename = "auto-accept-edits")]
    AutoAcceptEdits,
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "full-access", alias = "danger-full-access", alias = "workspace-write")]
    FullAccess,
}

impl RuntimeMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Supervised => "Supervised",
            Self::AutoAcceptEdits => "Auto-accept edits",
            Self::Auto => "Auto",
            Self::FullAccess => "Full access",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Supervised => "Ask before commands and file changes.",
            Self::AutoAcceptEdits => "Auto-approve edits, ask before other actions.",
            Self::Auto => "Supported providers approve routine actions; others still ask.",
            Self::FullAccess => "Allow commands and edits without prompts.",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalPolicy {
    #[serde(rename = "untrusted")]
    Untrusted,
    #[serde(rename = "on-failure")]
    OnFailure,
    #[serde(rename = "on-request")]
    OnRequest,
    #[serde(rename = "never")]
    Never,
}

impl ApprovalPolicy {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Untrusted => "Untrusted",
            Self::OnFailure => "On Failure",
            Self::OnRequest => "On Request",
            Self::Never => "Never (Auto)",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Claude,
    Codex,
    Cursor,
    Grok,
    OpenCode,
    Antigravity,
    Ollama,
    Custom,
}

impl ProviderKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex CLI",
            Self::Cursor => "Cursor CLI",
            Self::Grok => "Grok CLI",
            Self::OpenCode => "OpenCode",
            Self::Antigravity => "Antigravity (ACP)",
            Self::Ollama => "Local Ollama",
            Self::Custom => "Custom Agent",
        }
    }

    pub fn default_binary_name(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::Grok => "grok",
            Self::OpenCode => "opencode",
            Self::Antigravity => "antigravity",
            Self::Ollama => "ollama",
            Self::Custom => "k7-agent",
        }
    }

    pub fn available_models(&self) -> Vec<ModelOption> {
        match self {
            Self::Claude => vec![
                ModelOption::new("claude-3-7-sonnet", "Claude 3.7 Sonnet (Thinking)", true),
                ModelOption::new("claude-3-5-sonnet", "Claude 3.5 Sonnet", false),
                ModelOption::new("claude-3-5-haiku", "Claude 3.5 Haiku", false),
            ],
            Self::Codex => vec![
                ModelOption::new("codex-5.3", "Codex 5.3 (Reasoning)", true),
                ModelOption::new("gpt-4o", "GPT-4o", false),
                ModelOption::new("o3-mini", "o3-mini (High)", true),
            ],
            Self::Cursor => vec![
                ModelOption::new("cursor-small", "Cursor Fast", false),
                ModelOption::new("claude-3.5-sonnet", "Cursor Sonnet", false),
            ],
            Self::Grok => vec![
                ModelOption::new("grok-3", "Grok 3 (Thinking)", true),
                ModelOption::new("grok-2", "Grok 2", false),
            ],
            Self::OpenCode => vec![
                ModelOption::new("opencode-v1", "OpenCode DeepSeek R1", true),
                ModelOption::new("opencode-v1-lite", "OpenCode Qwen 2.5", false),
            ],
            Self::Antigravity => vec![
                ModelOption::new("gemini-3.8-flash-high", "Gemini 3.8 Flash High", true),
                ModelOption::new("gemini-2.5-pro", "Gemini 2.5 Pro", true),
            ],
            Self::Ollama => vec![
                ModelOption::new("qwen2.5-coder:14b", "Qwen 2.5 Coder 14B", false),
                ModelOption::new("deepseek-r1:14b", "DeepSeek R1 14B", true),
                ModelOption::new("llama3.3:70b", "Llama 3.3 70B", false),
            ],
            Self::Custom => vec![
                ModelOption::new("default", "Default Script Agent", false),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelOption {
    pub id: String,
    pub name: String,
    pub supports_thinking: bool,
}

impl ModelOption {
    pub fn new(id: impl Into<String>, name: impl Into<String>, supports_thinking: bool) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            supports_thinking,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    PendingApproval,
    Running,
    Completed,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolStep {
    pub id: String,
    pub tool_name: String,
    pub arguments: String,
    pub explanation: Option<String>,
    pub status: StepStatus,
    pub output: Option<String>,
    pub duration_ms: Option<u64>,
}

impl ToolStep {
    pub fn new(tool_name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            tool_name: tool_name.into(),
            arguments: arguments.into(),
            explanation: None,
            status: StepStatus::Running,
            output: None,
            duration_ms: None,
        }
    }

    pub fn with_approval(tool_name: impl Into<String>, arguments: impl Into<String>, explanation: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            tool_name: tool_name.into(),
            arguments: arguments.into(),
            explanation,
            status: StepStatus::PendingApproval,
            output: None,
            duration_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileDiffSummary {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnCheckpoint {
    pub id: String,
    pub git_sha: Option<String>,
    pub files_changed: Vec<FileDiffSummary>,
    pub diff_content: String,
    pub can_revert: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TurnStatus {
    Idle,
    Running,
    WaitingForApproval,
    Completed,
    Interrupted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Turn {
    pub id: String,
    pub turn_number: usize,
    pub user_prompt: String,
    pub assistant_response: String,
    pub steps: Vec<ToolStep>,
    pub checkpoint: Option<TurnCheckpoint>,
    pub status: TurnStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Turn {
    pub fn new(turn_number: usize, prompt: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            turn_number,
            user_prompt: prompt.into(),
            assistant_response: String::new(),
            steps: Vec::new(),
            checkpoint: None,
            status: TurnStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Thread {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub pinned: bool,
    pub archived: bool,
    pub runtime_mode: RuntimeMode,
    pub approval_policy: ApprovalPolicy,
    pub provider: ProviderKind,
    pub model: String,
    pub turns: Vec<Turn>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Thread {
    pub fn new(project_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            title: title.into(),
            pinned: false,
            archived: false,
            runtime_mode: RuntimeMode::AutoAcceptEdits,
            approval_policy: ApprovalPolicy::Untrusted,
            provider: ProviderKind::Claude,
            model: "claude-3-7-sonnet".to_string(),
            turns: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.turns
            .last()
            .map(|t| t.status == TurnStatus::Running || t.status == TurnStatus::WaitingForApproval)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub claude_path: Option<String>,
    pub codex_path: Option<String>,
    pub cursor_path: Option<String>,
    pub grok_path: Option<String>,
    pub opencode_path: Option<String>,
    pub antigravity_path: Option<String>,
    pub ollama_url: String,
    pub default_provider: ProviderKind,
    pub default_model: String,
    pub default_approval_policy: ApprovalPolicy,
    pub default_runtime_mode: RuntimeMode,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            claude_path: None,
            codex_path: None,
            cursor_path: None,
            grok_path: None,
            opencode_path: None,
            antigravity_path: None,
            ollama_url: "http://localhost:11434".to_string(),
            default_provider: ProviderKind::Claude,
            default_model: "claude-3-7-sonnet".to_string(),
            default_approval_policy: ApprovalPolicy::Untrusted,
            default_runtime_mode: RuntimeMode::AutoAcceptEdits,
            theme: "dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub projects: Vec<Project>,
    pub threads: Vec<Thread>,
    pub active_project_id: Option<String>,
    pub active_thread_id: Option<String>,
    pub settings: AppSettings,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            threads: Vec::new(),
            active_project_id: None,
            active_thread_id: None,
            settings: AppSettings::default(),
        }
    }
}
