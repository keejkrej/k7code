use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{StepStatus, Turn, TurnStatus};
use crate::ui::{h_flex, v_flex};

pub struct ChatRenderProps<'a> {
    pub turns: &'a [Turn],
    pub is_running: bool,
    pub project_name: &'a str,
    pub git_branch: Option<&'a str>,
    pub changed_files_count: usize,
}

pub struct ChatView;

impl ChatView {
    pub fn render<V: 'static>(
        props: ChatRenderProps<'_>,
        cx: &mut Context<V>,
        on_approve_step: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_reject_step: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_revert_checkpoint: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_starter_prompt: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_copy_text: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if props.turns.is_empty() {
            // Draft Hero Empty State matching T3 Code DraftHeroHeadline.tsx
            return v_flex()
                .flex_1()
                .w_full()
                .items_center()
                .justify_center()
                .p_8()
                .gap_6()
                .child(
                    v_flex()
                        .items_center()
                        .gap_3()
                        .child(
                            h_flex()
                                .items_center()
                                .gap_2()
                                .child(Icon::new(IconName::Sparkles).text_color(cx.theme().primary))
                                .child(
                                    div()
                                        .text_2xl()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(cx.theme().foreground)
                                        .child("Where should we start?")
                                )
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Select a starter task or type your instructions into the composer below.")
                        )
                        // Context Chips
                        .child(
                            h_flex()
                                .items_center()
                                .gap_2()
                                .pt_1()
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_1()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_full()
                                        .bg(cx.theme().secondary.opacity(0.4))
                                        .child(Icon::new(IconName::Folder).small().text_color(cx.theme().muted_foreground))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(props.project_name.to_string())
                                        )
                                )
                                .when_some(props.git_branch, |this, branch| {
                                    this.child(
                                        h_flex()
                                            .items_center()
                                            .gap_1()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_full()
                                            .bg(cx.theme().secondary.opacity(0.4))
                                            .child(Icon::new(IconName::GitBranch).small().text_color(cx.theme().muted_foreground))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(branch.to_string())
                                            )
                                    )
                                })
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_1()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_full()
                                        .bg(cx.theme().secondary.opacity(0.4))
                                        .child(Icon::new(IconName::GitCompare).small().text_color(cx.theme().muted_foreground))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(if props.changed_files_count == 0 {
                                                    "Working tree clean".to_string()
                                                } else {
                                                    format!("{} modified files", props.changed_files_count)
                                                })
                                        )
                                )
                        )
                )
                // 2x2 Starter Action Cards Grid
                .child(
                    v_flex()
                        .w_full()
                        .max_w(gpui::px(720.0))
                        .gap_3()
                        .child(
                            h_flex()
                                .gap_3()
                                .child(Self::starter_card(
                                    "starter-git",
                                    IconName::GitBranch,
                                    "Show Working Tree & Git Status",
                                    "Inspect branch, commits, and uncommitted file modifications",
                                    "git status",
                                    cx,
                                    on_starter_prompt.clone(),
                                ))
                                .child(Self::starter_card(
                                    "starter-check",
                                    IconName::Terminal,
                                    "Run Cargo Check",
                                    "Verify Rust compiler types, lints, and diagnostic errors",
                                    "run cargo check",
                                    cx,
                                    on_starter_prompt.clone(),
                                ))
                        )
                        .child(
                            h_flex()
                                .gap_3()
                                .child(Self::starter_card(
                                    "starter-arch",
                                    IconName::FolderTree,
                                    "Explain Project Architecture",
                                    "Summarize UI views, models, runner loop, and storage layout",
                                    "Explain the architecture of this project",
                                    cx,
                                    on_starter_prompt.clone(),
                                ))
                                .child(Self::starter_card(
                                    "starter-test",
                                    IconName::CheckCheck,
                                    "Run Project Tests",
                                    "Execute test suite to check functionality and regressions",
                                    "run cargo test",
                                    cx,
                                    on_starter_prompt.clone(),
                                ))
                        )
                )
                .into_any_element();
        }

        // Conversation Stream
        v_flex()
            .flex_1()
            .w_full()
            .overflow_hidden()
            .p_4()
            .gap_6()
            .children(props.turns.iter().map(|turn| {
                let turn_id = turn.id.clone();
                let user_prompt = turn.user_prompt.clone();
                let assistant_text = turn.assistant_response.clone();
                let is_turn_running = turn.status == TurnStatus::Running;
                let on_approve = on_approve_step.clone();
                let on_reject = on_reject_step.clone();
                let on_revert = on_revert_checkpoint.clone();
                let on_copy = on_copy_text.clone();

                v_flex()
                    .w_full()
                    .gap_4()
                    .child(
                        // User message card
                        h_flex()
                            .w_full()
                            .justify_end()
                            .child(
                                v_flex()
                                    .max_w(gpui::px(680.0))
                                    .p_3p5()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().secondary.opacity(0.45))
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .gap_2()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .child(Icon::new(IconName::User).small().text_color(cx.theme().primary))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .text_color(cx.theme().foreground)
                                                            .child("You")
                                                    )
                                                    .child(
                                                        Badge::new()
                                                            .small()
                                                            .child(format!("Turn #{}", turn.turn_number))
                                                    )
                                            )
                                            .child(
                                                Button::new(format!("copy-user-btn-{}", turn.id))
                                                    .ghost()
                                                    .small()
                                                    .icon(IconName::Copy)
                                                    .on_click(cx.listener({
                                                        let prompt_text = user_prompt.clone();
                                                        let on_copy = on_copy.clone();
                                                        move |this, _, window, cx| on_copy(this, prompt_text.clone(), window, cx)
                                                    }))
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().foreground)
                                            .child(user_prompt.clone())
                                    )
                            )
                    )
                    // Assistant response card
                    .child(
                        v_flex()
                            .w_full()
                            .p_4()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .gap_3()
                            .child(
                                // Assistant Header
                                h_flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child(Icon::new(IconName::Bot).small().text_color(cx.theme().primary))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(cx.theme().foreground)
                                                    .child("Assistant")
                                            )
                                            .when(is_turn_running, |this| {
                                                this.child(
                                                    Badge::new()
                                                        .small()
                                                        .child("Generating...")
                                                )
                                            })
                                    )
                                    .child(
                                        Button::new(format!("copy-asst-btn-{}", turn.id))
                                            .ghost()
                                            .small()
                                            .icon(IconName::Copy)
                                            .on_click(cx.listener({
                                                let asst_text = assistant_text.clone();
                                                let on_copy = on_copy.clone();
                                                move |this, _, window, cx| on_copy(this, asst_text.clone(), window, cx)
                                            }))
                                    )
                            )
                            // Assistant Text Output
                            .when(!turn.assistant_response.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_sm()
                                        .line_height(gpui::relative(1.4))
                                        .text_color(cx.theme().foreground)
                                        .child(turn.assistant_response.clone())
                                )
                            })
                            // Tool Execution Steps
                            .children(turn.steps.iter().map(|step| {
                                let step_id = step.id.clone();
                                let is_pending = step.status == StepStatus::PendingApproval;
                                let _is_running = step.status == StepStatus::Running;
                                let on_approve = on_approve.clone();
                                let on_reject = on_reject.clone();
                                let on_copy = on_copy.clone();
                                let command_text = step.arguments.clone();

                                v_flex()
                                    .p_3()
                                    .rounded(cx.theme().radius)
                                    .border_1()
                                    .border_color(if is_pending {
                                        gpui::rgb(0xf59e0b).into()
                                    } else {
                                        cx.theme().border
                                    })
                                    .bg(if is_pending {
                                        gpui::rgba(0xf59e0b10).into()
                                    } else {
                                        cx.theme().secondary.opacity(0.3)
                                    })
                                    .gap_2p5()
                                    // Step Header
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Icon::new(IconName::Terminal).small().text_color(cx.theme().primary))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .text_color(cx.theme().foreground)
                                                            .child(step.tool_name.clone())
                                                    )
                                            )
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .when_some(step.duration_ms, |this, ms| {
                                                        this.child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(cx.theme().muted_foreground)
                                                                .child(format!("{ms}ms"))
                                                        )
                                                    })
                                                    .child(
                                                        match step.status {
                                                            StepStatus::PendingApproval => Badge::new().small().child("Needs Approval"),
                                                            StepStatus::Running => Badge::new().small().child("Running..."),
                                                            StepStatus::Completed => Badge::new().small().child("Completed"),
                                                            StepStatus::Failed => Badge::new().small().child("Failed"),
                                                            StepStatus::Rejected => Badge::new().small().child("Rejected"),
                                                        }
                                                    )
                                            )
                                    )
                                    // Explanation text if any
                                    .when_some(step.explanation.as_ref(), |this, expl| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(expl.clone())
                                        )
                                    })
                                    // Monospace Command Box
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .justify_between()
                                            .p_2()
                                            .rounded(cx.theme().radius)
                                            .bg(gpui::black().opacity(0.3))
                                            .border_1()
                                            .border_color(cx.theme().border.opacity(0.4))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().primary)
                                                    .child(format!("$ {}", step.arguments))
                                            )
                                            .child(
                                                Button::new(format!("copy-cmd-{}", step.id))
                                                    .ghost()
                                                    .small()
                                                    .icon(IconName::Copy)
                                                    .on_click(cx.listener({
                                                        let cmd = command_text.clone();
                                                        let on_copy = on_copy.clone();
                                                        move |this, _, window, cx| on_copy(this, cmd.clone(), window, cx)
                                                    }))
                                            )
                                    )
                                    // Pending Approval Action Banner
                                    .when(is_pending, |this| {
                                        let step_id_approve = step_id.clone();
                                        let step_id_reject = step_id.clone();

                                        this.child(
                                            v_flex()
                                                .p_2p5()
                                                .rounded(cx.theme().radius)
                                                .bg(gpui::rgba(0xf59e0b15))
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .text_color(gpui::rgb(0xfbbf24))
                                                        .child("This tool proposal requires authorization before executing on your machine.")
                                                )
                                                .child(
                                                    h_flex()
                                                        .gap_2()
                                                        .child(
                                                            Button::new(format!("approve-{}", step_id))
                                                                .primary()
                                                                .small()
                                                                .icon(IconName::Check)
                                                                .label("Approve & Run")
                                                                .on_click(cx.listener({
                                                                    let on_approve = on_approve.clone();
                                                                    move |this, _, window, cx| on_approve(this, step_id_approve.clone(), window, cx)
                                                                }))
                                                        )
                                                        .child(
                                                            Button::new(format!("reject-{}", step_id))
                                                                .danger()
                                                                .small()
                                                                .icon(IconName::X)
                                                                .label("Reject")
                                                                .on_click(cx.listener({
                                                                    let on_reject = on_reject.clone();
                                                                    move |this, _, window, cx| on_reject(this, step_id_reject.clone(), window, cx)
                                                                }))
                                                        )
                                                )
                                        )
                                    })
                                    // Terminal output box
                                    .when_some(step.output.as_ref(), |this, out| {
                                        let output_text = out.clone();
                                        this.child(
                                            v_flex()
                                                .p_2()
                                                .rounded(cx.theme().radius)
                                                .bg(gpui::black().opacity(0.35))
                                                .border_1()
                                                .border_color(cx.theme().border.opacity(0.4))
                                                .max_h(gpui::px(200.0))
                                                .overflow_hidden()
                                                .gap_1()
                                                .child(
                                                    h_flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                                .text_color(cx.theme().muted_foreground)
                                                                .child("TERMINAL OUTPUT")
                                                        )
                                                        .child(
                                                            Button::new(format!("copy-out-{}", step.id))
                                                                .ghost()
                                                                .small()
                                                                .icon(IconName::Copy)
                                                                .on_click(cx.listener({
                                                                    let out_text = output_text.clone();
                                                                    let on_copy = on_copy.clone();
                                                                    move |this, _, window, cx| on_copy(this, out_text.clone(), window, cx)
                                                                }))
                                                        )
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().foreground)
                                                        .child(out.clone())
                                                )
                                        )
                                    })
                            }))
                            // Checkpoint & Revert Banner
                            .when_some(turn.checkpoint.as_ref(), |this, cp| {
                                let sha_short = cp.git_sha.as_deref().unwrap_or("snapshot").chars().take(7).collect::<String>();
                                let turn_id_revert = turn_id.clone();

                                this.child(
                                    h_flex()
                                        .items_center()
                                        .justify_between()
                                        .p_2p5()
                                        .rounded(cx.theme().radius)
                                        .bg(cx.theme().secondary.opacity(0.3))
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .child(
                                            h_flex()
                                                .items_center()
                                                .gap_2()
                                                .child(Icon::new(IconName::GitCompare).small().text_color(cx.theme().primary))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .text_color(cx.theme().foreground)
                                                        .child(format!("Checkpoint [{sha_short}]: {} files modified", cp.files_changed.len()))
                                                )
                                        )
                                        .child(
                                            Button::new(format!("revert-{}", turn.id))
                                                .secondary()
                                                .small()
                                                .icon(IconName::RotateCcw)
                                                .label("Revert Checkpoint")
                                                .on_click(cx.listener({
                                                    let on_revert = on_revert.clone();
                                                    move |this, _, window, cx| on_revert(this, turn_id_revert.clone(), window, cx)
                                                }))
                                        )
                                )
                            })
                    )
            }))
            .into_any_element()
    }

    fn starter_card<V: 'static>(
        _id: &'static str,
        icon: IconName,
        title: &'static str,
        description: &'static str,
        prompt: &'static str,
        cx: &mut Context<V>,
        on_starter_prompt: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        v_flex()
            .flex_1()
            .p_4()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary.opacity(0.25))
            .cursor_pointer()
            .gap_1p5()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    on_starter_prompt(this, prompt.to_string(), window, cx);
                }),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(Icon::new(icon).small().text_color(cx.theme().primary))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child(title)
                    )
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(description)
            )
    }
}
