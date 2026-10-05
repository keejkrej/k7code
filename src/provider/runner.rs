use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use async_channel::{Receiver, Sender};
use uuid::Uuid;

use crate::git::GitManager;
use crate::model::{
    ApprovalPolicy, ProviderKind, RuntimeMode, TurnCheckpoint,
};

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    TextDelta(String),
    ToolProposed {
        step_id: String,
        tool_name: String,
        args: String,
        explanation: Option<String>,
        requires_approval: bool,
    },
    ToolStarted {
        step_id: String,
    },
    ToolFinished {
        step_id: String,
        output: String,
        success: bool,
        duration_ms: u64,
    },
    TurnCompleted {
        checkpoint: Option<TurnCheckpoint>,
    },
    TurnFailed(String),
}

#[derive(Debug, Clone)]
pub enum RunnerCommand {
    ApproveTool { step_id: String },
    RejectTool { step_id: String },
    Cancel,
}

pub struct ExecutionContext {
    pub project_path: PathBuf,
    pub prompt: String,
    pub provider: ProviderKind,
    pub model: String,
    pub runtime_mode: RuntimeMode,
    pub approval_policy: ApprovalPolicy,
}

pub struct ProviderRunner {
    is_cancelled: Arc<AtomicBool>,
}

impl ProviderRunner {
    pub fn new() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }

    pub fn run_turn(
        &self,
        ctx: ExecutionContext,
        event_sender: Sender<ProviderEvent>,
        cmd_receiver: Receiver<RunnerCommand>,
    ) {
        let is_cancelled = Arc::clone(&self.is_cancelled);
        is_cancelled.store(false, Ordering::SeqCst);

        std::thread::spawn(move || {
            let start_head_sha = GitManager::current_head_sha(&ctx.project_path);

            let run_result = Self::execute_turn_loop(
                &ctx,
                &event_sender,
                &cmd_receiver,
                &is_cancelled,
            );

            if let Err(e) = run_result {
                let _ = event_sender.send_blocking(ProviderEvent::TurnFailed(e.to_string()));
                return;
            }

            let (files_changed, diff_content) =
                GitManager::get_diff_summary(&ctx.project_path, start_head_sha.as_deref());

            let checkpoint = if !diff_content.is_empty() || !files_changed.is_empty() {
                Some(TurnCheckpoint {
                    id: Uuid::new_v4().to_string(),
                    git_sha: start_head_sha,
                    files_changed,
                    diff_content,
                    can_revert: true,
                })
            } else {
                None
            };

            let _ = event_sender.send_blocking(ProviderEvent::TurnCompleted { checkpoint });
        });
    }

    fn execute_turn_loop(
        ctx: &ExecutionContext,
        event_sender: &Sender<ProviderEvent>,
        cmd_receiver: &Receiver<RunnerCommand>,
        is_cancelled: &Arc<AtomicBool>,
    ) -> Result<()> {
        let prompt = &ctx.prompt;

        let _ = event_sender.send_blocking(ProviderEvent::TextDelta(
            format!("Evaluating request using **{}** ({})\n\n", ctx.provider.display_name(), ctx.model),
        ));

        if is_cancelled.load(Ordering::SeqCst) {
            return Ok(());
        }

        let lower_prompt = prompt.to_lowercase();
        let is_git_status = lower_prompt.contains("git status") || lower_prompt.contains("branch");
        let is_cargo_check = lower_prompt.contains("cargo check") || lower_prompt.contains("build");
        let is_test = lower_prompt.contains("cargo test") || lower_prompt.contains("test");

        if is_git_status {
            Self::propose_and_exec_tool(
                "run_command",
                "git status",
                Some("Check current git workspace status".to_string()),
                ctx,
                event_sender,
                cmd_receiver,
                is_cancelled,
            )?;
        } else if is_cargo_check {
            Self::propose_and_exec_tool(
                "run_command",
                "cargo check",
                Some("Check project compilation".to_string()),
                ctx,
                event_sender,
                cmd_receiver,
                is_cancelled,
            )?;
        } else if is_test {
            Self::propose_and_exec_tool(
                "run_command",
                "cargo test",
                Some("Run project tests".to_string()),
                ctx,
                event_sender,
                cmd_receiver,
                is_cancelled,
            )?;
        } else if lower_prompt.starts_with("run ") || lower_prompt.starts_with("exec ") {
            let cmd = prompt.split_once(' ').map(|x| x.1).unwrap_or(prompt);
            Self::propose_and_exec_tool(
                "run_command",
                cmd,
                Some(format!("Execute command in project directory: {}", cmd)),
                ctx,
                event_sender,
                cmd_receiver,
                is_cancelled,
            )?;
        } else {
            let response = format!(
                "I analyzed your request: \"{}\"\n\n\
                * **Active Project**: `{}`\n\
                * **Runtime Mode**: `{}`\n\
                * **Approval Policy**: `{}`\n\n\
                Ready to assist with your codebase. You can ask me to run commands, inspect git status, edit files, or explain code.",
                prompt,
                ctx.project_path.display(),
                ctx.runtime_mode.label(),
                ctx.approval_policy.label()
            );

            for chunk in response.split_inclusive(' ') {
                if is_cancelled.load(Ordering::SeqCst) {
                    return Ok(());
                }
                let _ = event_sender.send_blocking(ProviderEvent::TextDelta(chunk.to_string()));
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        }

        Ok(())
    }

    fn propose_and_exec_tool(
        tool_name: &str,
        args: &str,
        explanation: Option<String>,
        ctx: &ExecutionContext,
        event_sender: &Sender<ProviderEvent>,
        cmd_receiver: &Receiver<RunnerCommand>,
        is_cancelled: &Arc<AtomicBool>,
    ) -> Result<()> {
        let step_id = Uuid::new_v4().to_string();

        let requires_approval = match ctx.approval_policy {
            ApprovalPolicy::Never => false,
            ApprovalPolicy::Untrusted => true,
            ApprovalPolicy::OnRequest | ApprovalPolicy::OnFailure => {
                ctx.runtime_mode != RuntimeMode::FullAccess
            }
        };

        let _ = event_sender.send_blocking(ProviderEvent::ToolProposed {
            step_id: step_id.clone(),
            tool_name: tool_name.to_string(),
            args: args.to_string(),
            explanation,
            requires_approval,
        });

        if requires_approval {
            loop {
                if is_cancelled.load(Ordering::SeqCst) {
                    return Ok(());
                }

                if let Ok(cmd) = cmd_receiver.recv_blocking() {
                    match cmd {
                        RunnerCommand::ApproveTool { step_id: approved_id } => {
                            if approved_id == step_id {
                                break;
                            }
                        }
                        RunnerCommand::RejectTool { step_id: rejected_id } => {
                            if rejected_id == step_id {
                                let _ = event_sender.send_blocking(ProviderEvent::ToolFinished {
                                    step_id,
                                    output: "Operation rejected by user.".to_string(),
                                    success: false,
                                    duration_ms: 0,
                                });
                                return Ok(());
                            }
                        }
                        RunnerCommand::Cancel => return Ok(()),
                    }
                }
            }
        }

        if is_cancelled.load(Ordering::SeqCst) {
            return Ok(());
        }

        let _ = event_sender.send_blocking(ProviderEvent::ToolStarted {
            step_id: step_id.clone(),
        });

        let start_time = Instant::now();

        let (output_str, success) = if tool_name == "run_command" {
            Self::execute_system_command(args, &ctx.project_path)
        } else {
            (format!("Unknown tool: {}", tool_name), false)
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        let _ = event_sender.send_blocking(ProviderEvent::ToolFinished {
            step_id,
            output: output_str,
            success,
            duration_ms,
        });

        Ok(())
    }

    fn execute_system_command(command: &str, project_dir: &std::path::Path) -> (String, bool) {
        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-Command", command]);
            c
        };

        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.args(["-c", command]);
            c
        };

        cmd.current_dir(project_dir);

        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let mut combined = String::new();
                if !stdout.is_empty() {
                    combined.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !combined.is_empty() {
                        combined.push_str("\n--- STDERR ---\n");
                    }
                    combined.push_str(&stderr);
                }
                (combined, output.status.success())
            }
            Err(e) => (format!("Failed to execute command: {}", e), false),
        }
    }
}
