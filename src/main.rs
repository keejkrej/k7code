pub mod git;
pub mod model;
pub mod provider;
pub mod storage;
pub mod ui;

use std::collections::HashMap;
use std::sync::Arc;

use async_channel::Sender;
use gpui_kit::component::{
    input::{InputEvent, InputState},
    ActiveTheme,
};
use gpui_kit::*;

use crate::git::GitManager;
use crate::model::{
    AppData, ApprovalPolicy, Project, ProviderKind, RuntimeMode, StepStatus, Thread, ToolStep,
    Turn, TurnStatus,
};
use crate::provider::{
    ExecutionContext, ProviderDetector, ProviderEvent, ProviderRunner, ProviderStatus,
    RunnerCommand,
};
use crate::storage::StorageManager;
use crate::ui::chat::{ChatRenderProps, ChatView};
use crate::ui::composer::{ComposerRenderProps, ComposerView};
use crate::ui::diff_panel::{DiffPanelRenderProps, DiffPanelView};
use crate::ui::header::{HeaderRenderProps, HeaderView};
use crate::ui::settings_modal::{SettingsRenderProps, SettingsModalView};
use crate::ui::sidebar::{SidebarRenderProps, SidebarView, ThreadFilterTab};
use crate::ui::v_flex;

pub struct K7AppView {
    pub data: AppData,
    pub storage: StorageManager,
    pub runner: Arc<ProviderRunner>,
    pub provider_statuses: HashMap<ProviderKind, ProviderStatus>,

    pub input_state: Entity<InputState>,
    pub filter_tab: ThreadFilterTab,
    pub is_diff_panel_open: bool,
    pub is_settings_open: bool,

    pub diff_content: String,
    pub git_branch: Option<String>,

    pub cmd_sender: Option<Sender<RunnerCommand>>,
}

impl K7AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let storage = StorageManager::new().expect("Failed to initialize storage");
        let data = storage.load_or_init();
        let provider_statuses = ProviderDetector::detect_all();
        let runner = Arc::new(ProviderRunner::new());

        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Ask a question, propose edits, or run a command (e.g. 'run cargo check')...")
        });

        let mut app = Self {
            data,
            storage,
            runner,
            provider_statuses,
            input_state: input_state.clone(),
            filter_tab: ThreadFilterTab::Active,
            is_diff_panel_open: false,
            is_settings_open: false,
            diff_content: String::new(),
            git_branch: None,
            cmd_sender: None,
        };

        app.refresh_git_status();

        // Subscribe to input enter event
        cx.subscribe_in(&input_state, window, |this, state, event, window, cx| {
            if let InputEvent::PressEnter { secondary: false, .. } = event {
                let prompt = state.read(cx).value().to_string();
                if !prompt.trim().is_empty() {
                    state.update(cx, |s, cx| s.set_value("", window, cx));
                    this.submit_prompt(prompt, window, cx);
                }
            }
        })
        .detach();

        app
    }

    pub fn refresh_git_status(&mut self) {
        let path = self.active_project().map(|p| p.path.clone());
        if let Some(p) = path {
            self.git_branch = GitManager::current_branch(&p);
            let (_files, diff) = GitManager::get_diff_summary(&p, None);
            self.diff_content = diff;
        }
    }

    pub fn active_project(&self) -> Option<&Project> {
        let pid = self.data.active_project_id.as_ref()?;
        self.data.projects.iter().find(|p| &p.id == pid)
    }

    pub fn active_thread(&self) -> Option<&Thread> {
        let tid = self.data.active_thread_id.as_ref()?;
        self.data.threads.iter().find(|t| &t.id == tid)
    }

    pub fn active_thread_mut(&mut self) -> Option<&mut Thread> {
        let tid = self.data.active_thread_id.as_ref()?;
        self.data.threads.iter_mut().find(|t| &t.id == tid)
    }

    pub fn save_state(&self) {
        let _ = self.storage.save(&self.data);
    }

    pub fn select_thread(&mut self, thread_id: String, _window: &mut Window, cx: &mut Context<Self>) {
        self.data.active_thread_id = Some(thread_id);
        self.refresh_git_status();
        self.save_state();
        cx.notify();
    }

    pub fn new_thread(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(pid) = self.data.active_project_id.clone() {
            let thread_count = self.data.threads.iter().filter(|t| t.project_id == pid).count();
            let new_t = Thread::new(&pid, format!("Thread #{}", thread_count + 1));
            let tid = new_t.id.clone();
            self.data.threads.insert(0, new_t);
            self.data.active_thread_id = Some(tid);
            self.refresh_git_status();
            self.save_state();
            cx.notify();
        }
    }

    pub fn toggle_pin(&mut self, thread_id: String, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = self.data.threads.iter_mut().find(|t| t.id == thread_id) {
            t.pinned = !t.pinned;
            self.save_state();
            cx.notify();
        }
    }

    pub fn toggle_archive(&mut self, thread_id: String, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = self.data.threads.iter_mut().find(|t| t.id == thread_id) {
            t.archived = !t.archived;
            self.save_state();
            cx.notify();
        }
    }

    pub fn delete_thread(&mut self, thread_id: String, _window: &mut Window, cx: &mut Context<Self>) {
        self.data.threads.retain(|t| t.id != thread_id);
        if self.data.active_thread_id.as_deref() == Some(&thread_id) {
            self.data.active_thread_id = self.data.threads.first().map(|t| t.id.clone());
        }
        self.refresh_git_status();
        self.save_state();
        cx.notify();
    }

    pub fn change_filter_tab(&mut self, tab: ThreadFilterTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.filter_tab = tab;
        cx.notify();
    }

    pub fn cycle_runtime_mode(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = self.active_thread_mut() {
            t.runtime_mode = match t.runtime_mode {
                RuntimeMode::ReadOnly => RuntimeMode::WorkspaceWrite,
                RuntimeMode::WorkspaceWrite => RuntimeMode::DangerFullAccess,
                RuntimeMode::DangerFullAccess => RuntimeMode::ReadOnly,
            };
            self.save_state();
            cx.notify();
        }
    }

    pub fn cycle_approval_policy(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = self.active_thread_mut() {
            t.approval_policy = match t.approval_policy {
                ApprovalPolicy::Untrusted => crate::model::ApprovalPolicy::OnRequest,
                ApprovalPolicy::OnRequest => crate::model::ApprovalPolicy::OnFailure,
                ApprovalPolicy::OnFailure => crate::model::ApprovalPolicy::Never,
                ApprovalPolicy::Never => crate::model::ApprovalPolicy::Untrusted,
            };
            self.save_state();
            cx.notify();
        }
    }

    pub fn cycle_model(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = self.active_thread_mut() {
            let providers = [
                ProviderKind::Claude,
                ProviderKind::Codex,
                ProviderKind::Cursor,
                ProviderKind::Grok,
                ProviderKind::OpenCode,
                ProviderKind::Antigravity,
                ProviderKind::Ollama,
            ];

            let curr_idx = providers.iter().position(|p| *p == t.provider).unwrap_or(0);
            let next_provider = providers[(curr_idx + 1) % providers.len()];
            let models = next_provider.available_models();
            let next_model = models.first().map(|m| m.id.clone()).unwrap_or_else(|| "default".to_string());

            t.provider = next_provider;
            t.model = next_model;
            self.save_state();
            cx.notify();
        }
    }

    pub fn submit_prompt(&mut self, prompt: String, _window: &mut Window, cx: &mut Context<Self>) {
        let (project_path, provider, model, runtime_mode, approval_policy) = {
            let proj = match self.active_project() {
                Some(p) => p,
                None => return,
            };
            let thr = match self.active_thread() {
                Some(t) => t,
                None => return,
            };
            (
                proj.path.clone(),
                thr.provider,
                thr.model.clone(),
                thr.runtime_mode,
                thr.approval_policy,
            )
        };

        // Create new turn in active thread
        let turn_num = self.active_thread().map(|t| t.turns.len() + 1).unwrap_or(1);
        let turn = Turn::new(turn_num, prompt.clone());

        if let Some(t) = self.active_thread_mut() {
            t.turns.push(turn);
        }
        self.save_state();
        cx.notify();

        // Setup execution channels
        let (event_sender, event_receiver) = async_channel::unbounded::<ProviderEvent>();
        let (cmd_sender, cmd_receiver) = async_channel::unbounded::<RunnerCommand>();
        self.cmd_sender = Some(cmd_sender);

        let exec_ctx = ExecutionContext {
            project_path,
            prompt,
            provider,
            model,
            runtime_mode,
            approval_policy,
        };

        let runner = Arc::clone(&self.runner);
        runner.run_turn(exec_ctx, event_sender, cmd_receiver);

        // Spawn background listener in GPUI executor to update UI as events stream in!
        cx.spawn(async move |this, cx| {
            while let Ok(event) = event_receiver.recv().await {
                let _ = this.update(cx, |view, cx| {
                    view.handle_provider_event(event);
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn handle_provider_event(&mut self, event: ProviderEvent) {
        let thr = match self.active_thread_mut() {
            Some(t) => t,
            None => return,
        };

        let turn = match thr.turns.last_mut() {
            Some(t) => t,
            None => return,
        };

        match event {
            ProviderEvent::TextDelta(delta) => {
                turn.assistant_response.push_str(&delta);
            }
            ProviderEvent::ToolProposed {
                step_id,
                tool_name,
                args,
                explanation,
                requires_approval,
            } => {
                let step = if requires_approval {
                    ToolStep::with_approval(tool_name, args, explanation)
                } else {
                    let mut s = ToolStep::new(tool_name, args);
                    s.explanation = explanation;
                    s
                };
                let mut step = step;
                step.id = step_id;
                turn.steps.push(step);
                if requires_approval {
                    turn.status = TurnStatus::WaitingForApproval;
                }
            }
            ProviderEvent::ToolStarted { step_id } => {
                if let Some(s) = turn.steps.iter_mut().find(|s| s.id == step_id) {
                    s.status = StepStatus::Running;
                }
                turn.status = TurnStatus::Running;
            }
            ProviderEvent::ToolFinished {
                step_id,
                output,
                success,
                duration_ms,
            } => {
                if let Some(s) = turn.steps.iter_mut().find(|s| s.id == step_id) {
                    s.status = if success {
                        StepStatus::Completed
                    } else {
                        StepStatus::Failed
                    };
                    s.output = Some(output);
                    s.duration_ms = Some(duration_ms);
                }
            }
            ProviderEvent::TurnCompleted { checkpoint } => {
                turn.checkpoint = checkpoint;
                turn.status = TurnStatus::Completed;
                turn.completed_at = Some(chrono::Utc::now());
                self.cmd_sender = None;
                self.refresh_git_status();
                self.save_state();
            }
            ProviderEvent::TurnFailed(err) => {
                turn.status = TurnStatus::Failed;
                turn.assistant_response.push_str(&format!("\n\n**Error**: {}", err));
                self.cmd_sender = None;
                self.save_state();
            }
        }
    }

    pub fn approve_tool(&mut self, step_id: String, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ref sender) = self.cmd_sender {
            let _ = sender.send_blocking(RunnerCommand::ApproveTool { step_id: step_id.clone() });
        }
        if let Some(thr) = self.active_thread_mut() {
            if let Some(turn) = thr.turns.last_mut() {
                if let Some(s) = turn.steps.iter_mut().find(|s| s.id == step_id) {
                    s.status = StepStatus::Running;
                }
                turn.status = TurnStatus::Running;
            }
        }
        cx.notify();
    }

    pub fn reject_tool(&mut self, step_id: String, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ref sender) = self.cmd_sender {
            let _ = sender.send_blocking(RunnerCommand::RejectTool { step_id: step_id.clone() });
        }
        if let Some(thr) = self.active_thread_mut() {
            if let Some(turn) = thr.turns.last_mut() {
                if let Some(s) = turn.steps.iter_mut().find(|s| s.id == step_id) {
                    s.status = StepStatus::Rejected;
                }
            }
        }
        cx.notify();
    }

    pub fn revert_turn(&mut self, turn_id: String, _window: &mut Window, cx: &mut Context<Self>) {
        let proj_path = match self.active_project() {
            Some(p) => p.path.clone(),
            None => return,
        };

        if let Some(thr) = self.active_thread_mut() {
            if let Some(turn) = thr.turns.iter_mut().find(|t| t.id == turn_id) {
                if let Some(ref cp) = turn.checkpoint {
                    if let Some(ref sha) = cp.git_sha {
                        let _ = GitManager::revert_to_sha(&proj_path, sha);
                        turn.checkpoint = None;
                    }
                }
            }
        }

        self.refresh_git_status();
        self.save_state();
        cx.notify();
    }

    pub fn revert_all_uncommitted(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(proj) = self.active_project() {
            if let Some(sha) = GitManager::current_head_sha(&proj.path) {
                let _ = GitManager::revert_to_sha(&proj.path, &sha);
            }
        }
        self.refresh_git_status();
        self.save_state();
        cx.notify();
    }

    pub fn cancel_turn(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.runner.cancel();
        if let Some(ref sender) = self.cmd_sender {
            let _ = sender.send_blocking(RunnerCommand::Cancel);
        }
        if let Some(thr) = self.active_thread_mut() {
            if let Some(turn) = thr.turns.last_mut() {
                if turn.status == TurnStatus::Running || turn.status == TurnStatus::WaitingForApproval {
                    turn.status = TurnStatus::Interrupted;
                }
            }
        }
        self.cmd_sender = None;
        self.save_state();
        cx.notify();
    }
}

impl Render for K7AppView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (active_thread, is_running, turns_slice, provider, model) = match self.active_thread() {
            Some(t) => (
                Some(t),
                t.is_running(),
                t.turns.as_slice(),
                t.provider,
                t.model.as_str(),
            ),
            None => (None, false, [].as_slice(), ProviderKind::Claude, "claude-3-7-sonnet"),
        };

        let changed_files_count = if !self.diff_content.is_empty() { 1 } else { 0 };

        div()
            .flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                // 1. Left Sidebar
                SidebarView::render(
                    SidebarRenderProps {
                        projects: &self.data.projects,
                        active_project: self.active_project(),
                        threads: &self.data.threads,
                        active_thread_id: self.data.active_thread_id.as_deref(),
                        selected_tab: self.filter_tab,
                        detected_providers_count: self.provider_statuses.values().filter(|s| s.is_available).count(),
                    },
                    cx,
                    |this, tid, window, cx| this.select_thread(tid, window, cx),
                    |this, window, cx| this.new_thread(window, cx),
                    |this, tid, window, cx| this.toggle_pin(tid, window, cx),
                    |this, tid, window, cx| this.toggle_archive(tid, window, cx),
                    |this, tid, window, cx| this.delete_thread(tid, window, cx),
                    |this, tab, window, cx| this.change_filter_tab(tab, window, cx),
                    |this, _window, cx| {
                        this.is_settings_open = true;
                        cx.notify();
                    },
                )
            )
            .child(
                // 2. Center Main View: Header + Conversation / Empty + Composer
                v_flex()
                    .flex_1()
                    .h_full()
                    .justify_between()
                    .child(
                        HeaderView::render(
                            HeaderRenderProps {
                                active_thread,
                                git_branch: self.git_branch.as_deref(),
                                changed_files_count,
                                is_diff_panel_open: self.is_diff_panel_open,
                            },
                            cx,
                            |this, _window, cx| {
                                this.is_diff_panel_open = !this.is_diff_panel_open;
                                cx.notify();
                            },
                            |this, window, cx| this.cycle_runtime_mode(window, cx),
                            |this, window, cx| this.cycle_approval_policy(window, cx),
                        )
                    )
                    .child(
                        ChatView::render(
                            ChatRenderProps {
                                turns: turns_slice,
                                is_running,
                            },
                            cx,
                            |this, sid, window, cx| this.approve_tool(sid, window, cx),
                            |this, sid, window, cx| this.reject_tool(sid, window, cx),
                            |this, tid, window, cx| this.revert_turn(tid, window, cx),
                            |this, prompt, window, cx| this.submit_prompt(prompt, window, cx),
                        )
                    )
                    .child(
                        ComposerView::render(
                            ComposerRenderProps {
                                input_state: &self.input_state,
                                provider,
                                model,
                                is_running,
                            },
                            cx,
                            |this, window, cx| {
                                let prompt = this.input_state.read(cx).value().to_string();
                                if !prompt.trim().is_empty() {
                                    this.input_state.update(cx, |s, cx| s.set_value("", window, cx));
                                    this.submit_prompt(prompt, window, cx);
                                }
                            },
                            |this, window, cx| this.cancel_turn(window, cx),
                            |this, window, cx| this.cycle_model(window, cx),
                        )
                    )
            )
            .child(
                // 3. Right Diff Panel
                DiffPanelView::render(
                    DiffPanelRenderProps {
                        files: &[],
                        diff_content: &self.diff_content,
                        is_open: self.is_diff_panel_open,
                    },
                    cx,
                    |this, _window, cx| {
                        this.is_diff_panel_open = false;
                        cx.notify();
                    },
                    |this, window, cx| this.revert_all_uncommitted(window, cx),
                )
            )
            .child(
                // 4. Modal Overlay for Settings
                SettingsModalView::render(
                    SettingsRenderProps {
                        is_open: self.is_settings_open,
                        settings: &self.data.settings,
                        provider_statuses: &self.provider_statuses,
                    },
                    cx,
                    |this, _window, cx| {
                        this.is_settings_open = false;
                        cx.notify();
                    },
                )
            )
    }
}

fn main() {
    application()
        .with_assets(assets::Assets)
        .run(|cx| {
            init(cx);
            open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("k7code - Rust GPUI Agent Workspace".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    window_bounds: Some(WindowBounds::Windowed(gpui::Bounds {
                        origin: gpui::Point::new(gpui::px(100.0), gpui::px(100.0)),
                        size: gpui::Size {
                            width: gpui::px(1280.0),
                            height: gpui::px(820.0),
                        },
                    })),
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| K7AppView::new(window, cx)),
            )
            .expect("Failed to open main window");
        });
}
