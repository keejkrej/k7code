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
    AppData, ApprovalPolicy, FileDiffSummary, Project, ProviderKind, RuntimeMode, StepStatus,
    Thread, ToolStep, Turn, TurnStatus,
};
use crate::provider::{
    ExecutionContext, ProviderDetector, ProviderEvent, ProviderRunner, ProviderStatus,
    RunnerCommand,
};
use crate::storage::StorageManager;
use crate::ui::chat::{ChatRenderProps, ChatView};
use crate::ui::composer::{ComposerRenderProps, ComposerView};
use crate::ui::header::{HeaderRenderProps, HeaderView};
use crate::ui::model_picker::{ModelPickerModalView, ModelPickerRenderProps};
use crate::ui::rename_modal::{RenameModalRenderProps, RenameModalView};
use crate::ui::right_panel::{RightPanelRenderProps, RightPanelTab, RightPanelView};
use crate::ui::settings_modal::{SettingsModalView, SettingsRenderProps, SettingsTab};
use crate::ui::sidebar::{SidebarFilter, SidebarRenderProps, SidebarView};
use crate::ui::v_flex;

pub struct K7AppView {
    pub data: AppData,
    pub storage: StorageManager,
    pub runner: Arc<ProviderRunner>,
    pub provider_statuses: HashMap<ProviderKind, ProviderStatus>,

    pub input_state: Entity<InputState>,
    pub search_input_state: Entity<InputState>,
    pub rename_input_state: Entity<InputState>,

    pub is_sidebar_open: bool,
    pub sidebar_filter: SidebarFilter,

    pub is_right_panel_open: bool,
    pub active_right_tab: RightPanelTab,
    pub diff_files: Vec<FileDiffSummary>,
    pub diff_content: String,
    pub selected_diff_file: Option<String>,
    pub terminal_logs: Vec<String>,
    pub linked_pr_number: Option<usize>,
    pub linked_pr_url: Option<String>,
    pub linked_pr_title: Option<String>,
    pub used_tokens: usize,
    pub max_tokens: usize,

    pub is_settings_open: bool,
    pub settings_tab: SettingsTab,

    pub is_model_picker_open: bool,
    pub is_rename_open: bool,
    pub renaming_thread_id: Option<String>,

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
                .placeholder("Ask a question, propose changes, or run tasks (e.g. 'cargo check')...")
        });

        let search_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search threads or prompts...")
        });

        let rename_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter new thread title...")
        });

        let mut app = Self {
            data,
            storage,
            runner,
            provider_statuses,
            input_state: input_state.clone(),
            search_input_state,
            rename_input_state,
            is_sidebar_open: true,
            sidebar_filter: SidebarFilter::Active,
            is_right_panel_open: false,
            active_right_tab: RightPanelTab::Diffs,
            diff_files: Vec::new(),
            diff_content: String::new(),
            selected_diff_file: None,
            terminal_logs: vec![
                "[00:00:00] Antigravity CLI runner v0.9.4 initialized".to_string(),
                "[00:00:01] Workspace bound to C:\\Users\\ctyja\\workspace\\k7code".to_string(),
                "[00:00:02] Git repository: keejkrej/k7code on feat/t3code-ui-parity".to_string(),
                "[00:00:02] PR #2 linked: feat(ui): complete section-by-section parity with T3 Code".to_string(),
            ],
            linked_pr_number: Some(2),
            linked_pr_url: Some("https://github.com/keejkrej/k7code/pull/2".to_string()),
            linked_pr_title: Some("feat(ui): complete section-by-section parity with T3 Code".to_string()),
            used_tokens: 14200,
            max_tokens: 200000,
            is_settings_open: false,
            settings_tab: SettingsTab::Providers,
            is_model_picker_open: false,
            is_rename_open: false,
            renaming_thread_id: None,
            git_branch: None,
            cmd_sender: None,
        };

        app.refresh_git_status();

        // Subscribe to composer input enter event
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
            if let Some(ref file) = self.selected_diff_file {
                let (all_files, _) = GitManager::get_diff_summary(&p, None);
                let file_diff = GitManager::get_file_diff(&p, file);
                self.diff_files = all_files;
                self.diff_content = file_diff;
            } else {
                let (files, diff) = GitManager::get_diff_summary(&p, None);
                self.diff_files = files;
                self.diff_content = diff;
            }
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
        self.selected_diff_file = None;
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
            self.selected_diff_file = None;
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

    pub fn change_filter_tab(&mut self, filter: SidebarFilter, _window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_filter = filter;
        cx.notify();
    }

    pub fn open_rename_modal(&mut self, thread_id: String, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(thread) = self.data.threads.iter().find(|t| t.id == thread_id) {
            let title = thread.title.clone();
            self.rename_input_state.update(cx, |s, cx| {
                s.set_value(&title, window, cx);
            });
            self.renaming_thread_id = Some(thread_id);
            self.is_rename_open = true;
            cx.notify();
        }
    }

    pub fn save_rename_modal(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(thread_id) = self.renaming_thread_id.take() {
            let new_title = self.rename_input_state.read(cx).value().trim().to_string();
            if !new_title.is_empty() {
                if let Some(thread) = self.data.threads.iter_mut().find(|t| t.id == thread_id) {
                    thread.title = new_title;
                    thread.updated_at = chrono::Utc::now();
                    self.save_state();
                }
            }
        }
        self.is_rename_open = false;
        cx.notify();
    }

    pub fn cycle_runtime_mode(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = self.active_thread_mut() {
            t.runtime_mode = match t.runtime_mode {
                RuntimeMode::Supervised => RuntimeMode::AutoAcceptEdits,
                RuntimeMode::AutoAcceptEdits => RuntimeMode::Auto,
                RuntimeMode::Auto => RuntimeMode::FullAccess,
                RuntimeMode::FullAccess => RuntimeMode::Supervised,
            };
            self.save_state();
            cx.notify();
        }
    }

    pub fn cycle_approval_policy(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = self.active_thread_mut() {
            t.approval_policy = match t.approval_policy {
                ApprovalPolicy::Untrusted => ApprovalPolicy::OnRequest,
                ApprovalPolicy::OnRequest => ApprovalPolicy::OnFailure,
                ApprovalPolicy::OnFailure => ApprovalPolicy::Never,
                ApprovalPolicy::Never => ApprovalPolicy::Untrusted,
            };
            self.save_state();
            cx.notify();
        }
    }

    pub fn select_provider_and_model(
        &mut self,
        provider: ProviderKind,
        model: String,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(t) = self.active_thread_mut() {
            t.provider = provider;
            t.model = model;
            t.updated_at = chrono::Utc::now();
            self.save_state();
        }
        self.is_model_picker_open = false;
        cx.notify();
    }

    pub fn select_diff_file(&mut self, file: Option<String>, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_diff_file = file;
        self.refresh_git_status();
        cx.notify();
    }

    pub fn rescan_providers(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.provider_statuses = ProviderDetector::detect_all();
        cx.notify();
    }

    pub fn copy_to_clipboard(&mut self, text: String, _window: &mut Window, cx: &mut Context<Self>) {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
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
        self.used_tokens += prompt.len() / 4 + 150;
        self.terminal_logs.push(format!("[turn #{}] User prompt: {}", turn_num, prompt));
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

        // Spawn background listener in GPUI executor to update UI as events stream in
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
                self.used_tokens += delta.len() / 4;
            }
            ProviderEvent::ToolProposed {
                step_id,
                tool_name,
                args,
                explanation,
                requires_approval,
            } => {
                let step = if requires_approval {
                    ToolStep::with_approval(tool_name.clone(), args.clone(), explanation)
                } else {
                    let mut s = ToolStep::new(tool_name.clone(), args.clone());
                    s.explanation = explanation;
                    s
                };
                let mut step = step;
                step.id = step_id.clone();
                turn.steps.push(step);
                if requires_approval {
                    turn.status = TurnStatus::WaitingForApproval;
                }
                self.terminal_logs.push(format!("[exec] Proposed tool: {} ({})", tool_name, args));
            }
            ProviderEvent::ToolStarted { step_id } => {
                if let Some(s) = turn.steps.iter_mut().find(|s| s.id == step_id) {
                    s.status = StepStatus::Running;
                }
                turn.status = TurnStatus::Running;
                self.terminal_logs.push(format!("[exec] Started tool execution: {}", step_id));
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
                    s.output = Some(output.clone());
                    s.duration_ms = Some(duration_ms);
                }
                self.terminal_logs.push(format!("[exec] Finished tool in {}ms (success={})", duration_ms, success));
            }
            ProviderEvent::TurnCompleted { checkpoint } => {
                turn.checkpoint = checkpoint;
                turn.status = TurnStatus::Completed;
                turn.completed_at = Some(chrono::Utc::now());
                self.cmd_sender = None;
                self.terminal_logs.push("[turn] Completed successfully.".to_string());
                self.refresh_git_status();
                self.save_state();
            }
            ProviderEvent::TurnFailed(err) => {
                turn.status = TurnStatus::Failed;
                turn.assistant_response.push_str(&format!("\n\n**Error**: {}", err));
                self.terminal_logs.push(format!("[turn] Failed with error: {}", err));
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
        self.selected_diff_file = None;
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
        let (active_thread, is_running, turns_slice, provider, model, runtime_mode, approval_policy) = match self.active_thread() {
            Some(t) => (
                Some(t),
                t.is_running(),
                t.turns.as_slice(),
                t.provider,
                t.model.as_str(),
                t.runtime_mode,
                t.approval_policy,
            ),
            None => (
                None,
                false,
                [].as_slice(),
                ProviderKind::Claude,
                "claude-3-7-sonnet",
                RuntimeMode::Supervised,
                ApprovalPolicy::Untrusted,
            ),
        };

        let active_project_name = self
            .active_project()
            .map(|p| p.name.as_str())
            .unwrap_or("k7code");

        let active_thread_title = active_thread
            .map(|t| t.title.as_str())
            .unwrap_or("New Thread");

        let active_thread_id_for_rename = self.data.active_thread_id.clone();
        let search_query = self.search_input_state.read(cx).value().to_string();

        div()
            .flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                // 1. Left Collapsible Sidebar
                if self.is_sidebar_open {
                    SidebarView::render(
                        SidebarRenderProps {
                            projects: &self.data.projects,
                            active_project: self.active_project(),
                            threads: &self.data.threads,
                            active_thread_id: self.data.active_thread_id.as_deref(),
                            active_provider: provider,
                            available_providers_count: self.provider_statuses.values().filter(|s| s.is_available).count(),
                            current_filter: self.sidebar_filter,
                            search_input_state: &self.search_input_state,
                            search_query: &search_query,
                        },
                        cx,
                        |this, tid, window, cx| this.select_thread(tid, window, cx),
                        |this, window, cx| this.new_thread(window, cx),
                        |this, filter, window, cx| this.change_filter_tab(filter, window, cx),
                        |this, tid, window, cx| this.toggle_pin(tid, window, cx),
                        |this, tid, window, cx| this.toggle_archive(tid, window, cx),
                        |this, tid, window, cx| this.open_rename_modal(tid, window, cx),
                        |this, tid, window, cx| this.delete_thread(tid, window, cx),
                        |this, _window, cx| {
                            this.is_settings_open = true;
                            cx.notify();
                        },
                    ).into_any_element()
                } else {
                    div().into_any_element()
                }
            )
            .child(
                // 2. Center Main View: Header + Conversation / Empty + Floating Composer
                v_flex()
                    .flex_1()
                    .h_full()
                    .justify_between()
                    .child(
                        HeaderView::render(
                            HeaderRenderProps {
                                is_sidebar_open: self.is_sidebar_open,
                                is_right_panel_open: self.is_right_panel_open,
                                active_right_tab: self.active_right_tab,
                                project_name: active_project_name,
                                git_branch: self.git_branch.as_deref(),
                                thread_title: Some(active_thread_title),
                                model,
                                runtime_mode,
                                approval_policy,
                                changed_files_count: self.diff_files.len(),
                                linked_pr_number: self.linked_pr_number,
                            },
                            cx,
                            |this, _window, cx| {
                                this.is_sidebar_open = !this.is_sidebar_open;
                                cx.notify();
                            },
                            |this, tab, _window, cx| {
                                if this.is_right_panel_open && this.active_right_tab == tab {
                                    this.is_right_panel_open = false;
                                } else {
                                    this.is_right_panel_open = true;
                                    this.active_right_tab = tab;
                                }
                                cx.notify();
                            },
                            |this, window, cx| this.new_thread(window, cx),
                            {
                                let thread_id_opt = active_thread_id_for_rename;
                                move |this, window, cx| {
                                    if let Some(ref tid) = thread_id_opt {
                                        this.open_rename_modal(tid.clone(), window, cx);
                                    }
                                }
                            },
                            |this, window, cx| this.cycle_runtime_mode(window, cx),
                            |this, window, cx| this.cycle_approval_policy(window, cx),
                            |this, _window, cx| {
                                this.is_settings_open = true;
                                cx.notify();
                            },
                        )
                    )
                    .child(
                        ChatView::render(
                            ChatRenderProps {
                                turns: turns_slice,
                                is_running,
                                project_name: active_project_name,
                                git_branch: self.git_branch.as_deref(),
                                changed_files_count: self.diff_files.len(),
                            },
                            cx,
                            |this, sid, window, cx| this.approve_tool(sid, window, cx),
                            |this, sid, window, cx| this.reject_tool(sid, window, cx),
                            |this, tid, window, cx| this.revert_turn(tid, window, cx),
                            |this, prompt, window, cx| this.submit_prompt(prompt, window, cx),
                            |this, text, window, cx| this.copy_to_clipboard(text, window, cx),
                        )
                    )
                    .child(
                        ComposerView::render(
                            ComposerRenderProps {
                                input_state: &self.input_state,
                                provider,
                                model,
                                runtime_mode,
                                project_name: active_project_name,
                                git_branch: self.git_branch.as_deref(),
                                is_running,
                                used_tokens: self.used_tokens,
                                max_tokens: self.max_tokens,
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
                            |this, _window, cx| {
                                this.is_model_picker_open = true;
                                cx.notify();
                            },
                            |this, window, cx| {
                                this.input_state.update(cx, |s, cx| s.set_value("", window, cx));
                                cx.notify();
                            },
                        )
                    )
            )
            .child(
                // 3. Multi-Tab Right Panel (Diffs / Terminal / Pull Requests)
                RightPanelView::render(
                    RightPanelRenderProps {
                        is_open: self.is_right_panel_open,
                        active_tab: self.active_right_tab,
                        diff_content: &self.diff_content,
                        diff_files: &self.diff_files,
                        selected_file: self.selected_diff_file.as_deref(),
                        changed_files_count: self.diff_files.len(),
                        terminal_logs: &self.terminal_logs,
                        linked_pr_number: self.linked_pr_number,
                        linked_pr_url: self.linked_pr_url.as_deref(),
                        linked_pr_title: self.linked_pr_title.as_deref(),
                    },
                    cx,
                    |this, tab, _window, cx| {
                        this.active_right_tab = tab;
                        cx.notify();
                    },
                    |this, file_opt, window, cx| this.select_diff_file(file_opt, window, cx),
                    |this, _window, cx| {
                        this.refresh_git_status();
                        cx.notify();
                    },
                    |this, window, cx| this.revert_all_uncommitted(window, cx),
                    |this, text, window, cx| this.copy_to_clipboard(text, window, cx),
                    |this, _window, cx| {
                        this.is_right_panel_open = false;
                        cx.notify();
                    },
                )
            )
            // 4. Overlays & Dialogs
            .child(
                // Model & Provider Picker Dialog
                ModelPickerModalView::render(
                    ModelPickerRenderProps {
                        is_open: self.is_model_picker_open,
                        current_provider: provider,
                        current_model: model,
                    },
                    cx,
                    |this, prov, mdl, window, cx| this.select_provider_and_model(prov, mdl, window, cx),
                    |this, _window, cx| {
                        this.is_model_picker_open = false;
                        cx.notify();
                    },
                )
            )
            .child(
                // Thread Rename Dialog
                RenameModalView::render(
                    RenameModalRenderProps {
                        is_open: self.is_rename_open,
                        input_state: &self.rename_input_state,
                    },
                    cx,
                    |this, window, cx| this.save_rename_modal(window, cx),
                    |this, _window, cx| {
                        this.is_rename_open = false;
                        cx.notify();
                    },
                )
            )
            .child(
                // Settings & Configuration Modal
                SettingsModalView::render(
                    SettingsRenderProps {
                        is_open: self.is_settings_open,
                        current_tab: self.settings_tab,
                        app_data: &self.data,
                        provider_statuses: &self.provider_statuses,
                    },
                    cx,
                    |this, tab, _window, cx| {
                        this.settings_tab = tab;
                        cx.notify();
                    },
                    |this, window, cx| this.rescan_providers(window, cx),
                    |this, mode, _window, cx| {
                        this.data.settings.default_runtime_mode = mode;
                        this.save_state();
                        cx.notify();
                    },
                    |this, policy, _window, cx| {
                        this.data.settings.default_approval_policy = policy;
                        this.save_state();
                        cx.notify();
                    },
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
                        title: Some("T3 Code (k7code - Pure Rust Native)".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    window_bounds: Some(WindowBounds::Windowed(gpui::Bounds {
                        origin: gpui::Point::new(gpui::px(100.0), gpui::px(100.0)),
                        size: gpui::Size {
                            width: gpui::px(1280.0),
                            height: gpui::px(840.0),
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
