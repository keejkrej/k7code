use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{StepStatus, ToolStep, Turn, TurnStatus};
use crate::ui::{h_flex, v_flex};

pub struct ChatRenderProps<'a> {
    pub turns: &'a [Turn],
    pub is_running: bool,
}

pub struct ChatView;

impl ChatView {
    pub fn render<V: 'static>(
        props: ChatRenderProps<'_>,
        cx: &mut Context<V>,
        on_approve_tool: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_reject_tool: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_revert_turn: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_starter_prompt: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if props.turns.is_empty() {
            return Self::render_empty_state(cx, on_starter_prompt).into_any_element();
        }

        let mut turn_elements = Vec::new();
        for turn in props.turns {
            turn_elements.push(
                Self::render_turn(
                    turn,
                    cx,
                    on_approve_tool.clone(),
                    on_reject_tool.clone(),
                    on_revert_turn.clone(),
                )
                .into_any_element(),
            );
        }

        v_flex()
            .flex_1()
            .w_full()
            .overflow_hidden()
            .items_center()
            .p_4()
            .gap_4()
            .child(
                v_flex()
                    .w_full()
                    .max_w(gpui::px(860.0))
                    .gap_4()
                    .children(turn_elements)
            )
            .into_any_element()
    }

    fn render_empty_state<V: 'static>(
        cx: &mut Context<V>,
        on_starter_prompt: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        v_flex()
            .flex_1()
            .w_full()
            .items_center()
            .justify_center()
            .p_8()
            .gap_8()
            .child(
                // Draft Hero Headline matching T3 Code DraftHeroHeadline.tsx
                v_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Icon::new(IconName::Sparkles)
                                    .small()
                                    .text_color(cx.theme().primary)
                            )
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
            )
            // 2x2 Starter Cards Grid matching T3 Code
            .child(
                v_flex()
                    .max_w(gpui::px(680.0))
                    .w_full()
                    .gap_3()
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .child(
                                Button::new("starter-git")
                                    .secondary()
                                    .flex_1()
                                    .icon(IconName::GitBranch)
                                    .label("Show Working Tree & Git Status")
                                    .on_click(cx.listener({
                                        let on_starter_prompt = on_starter_prompt.clone();
                                        move |this, _, window, cx| {
                                            on_starter_prompt(this, "git status and show modified files".to_string(), window, cx);
                                        }
                                    }))
                            )
                            .child(
                                Button::new("starter-check")
                                    .secondary()
                                    .flex_1()
                                    .icon(IconName::Terminal)
                                    .label("Run Cargo Check")
                                    .on_click(cx.listener({
                                        let on_starter_prompt = on_starter_prompt.clone();
                                        move |this, _, window, cx| {
                                            on_starter_prompt(this, "run cargo check".to_string(), window, cx);
                                        }
                                    }))
                            )
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .child(
                                Button::new("starter-arch")
                                    .secondary()
                                    .flex_1()
                                    .icon(IconName::Blocks)
                                    .label("Explain Project Architecture")
                                    .on_click(cx.listener({
                                        let on_starter_prompt = on_starter_prompt.clone();
                                        move |this, _, window, cx| {
                                            on_starter_prompt(this, "summarize project architecture and core modules".to_string(), window, cx);
                                        }
                                    }))
                            )
                            .child(
                                Button::new("starter-test")
                                    .secondary()
                                    .flex_1()
                                    .icon(IconName::Check)
                                    .label("Run Test Suite")
                                    .on_click(cx.listener({
                                        let on_starter_prompt = on_starter_prompt.clone();
                                        move |this, _, window, cx| {
                                            on_starter_prompt(this, "run cargo test".to_string(), window, cx);
                                        }
                                    }))
                            )
                    )
            )
    }

    fn render_turn<V: 'static>(
        turn: &Turn,
        cx: &mut Context<V>,
        on_approve_tool: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_reject_tool: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_revert_turn: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let is_running = turn.status == TurnStatus::Running || turn.status == TurnStatus::WaitingForApproval;
        let cid = turn.id.clone();

        let mut step_elements = Vec::new();
        for step in &turn.steps {
            step_elements.push(
                Self::render_tool_step(step, cx, on_approve_tool.clone(), on_reject_tool.clone())
                    .into_any_element(),
            );
        }

        v_flex()
            .w_full()
            .gap_3()
            // 1. User Prompt Bubble (right-aligned card)
            .child(
                h_flex()
                    .w_full()
                    .justify_end()
                    .child(
                        v_flex()
                            .max_w(gpui::px(640.0))
                            .p_3p5()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().accent.opacity(0.15))
                            .border_1()
                            .border_color(cx.theme().primary.opacity(0.35))
                            .gap_1()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Icon::new(IconName::User).small().text_color(cx.theme().primary))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().primary)
                                            .child(format!("You · Turn #{}", turn.turn_number))
                                    )
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child(turn.user_prompt.clone())
                            )
                    )
            )
            // 2. Tool Execution Steps (cards, approvals, terminal outputs)
            .when(!turn.steps.is_empty(), |this| {
                this.child(
                    v_flex()
                        .w_full()
                        .gap_2p5()
                        .children(step_elements)
                )
            })
            // 3. Assistant Response Bubble
            .when(!turn.assistant_response.is_empty() || is_running, |this| {
                this.child(
                    v_flex()
                        .w_full()
                        .p_4()
                        .rounded(cx.theme().radius)
                        .bg(cx.theme().secondary.opacity(0.25))
                        .border_1()
                        .border_color(cx.theme().border)
                        .gap_2()
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
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(turn.assistant_response.clone())
                        )
                        .when(is_running, |this| {
                            this.child(
                                h_flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(Icon::new(IconName::RotateCw).small().text_color(cx.theme().primary))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Executing operations...")
                                    )
                            )
                        })
                )
            })
            // 4. Git Checkpoint & Revert Banner matching T3 Code CheckpointCard
            .when(turn.checkpoint.is_some(), |this| {
                let cp = turn.checkpoint.as_ref().unwrap();
                let sha = cp.git_sha.as_deref().unwrap_or("unknown");
                let files_count = cp.files_changed.len();

                this.child(
                    h_flex()
                        .w_full()
                        .p_2p5()
                        .rounded(cx.theme().radius)
                        .bg(cx.theme().secondary.opacity(0.35))
                        .border_1()
                        .border_color(cx.theme().border)
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex()
                                .items_center()
                                .gap_2()
                                .child(Icon::new(IconName::GitCommitHorizontal).small().text_color(cx.theme().primary))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("Checkpoint: {} ({} files affected)", sha, files_count))
                                )
                        )
                        .child(
                            Button::new(format!("revert-{}", cid))
                                .ghost()
                                .small()
                                .icon(IconName::RotateCcw)
                                .label("Revert to Checkpoint")
                                .on_click(cx.listener({
                                    let on_revert_turn = on_revert_turn.clone();
                                    move |this, _, window, cx| on_revert_turn(this, cid.clone(), window, cx)
                                }))
                        )
                )
            })
    }

    fn render_tool_step<V: 'static>(
        step: &ToolStep,
        cx: &mut Context<V>,
        on_approve_tool: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_reject_tool: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let sid_app = step.id.clone();
        let sid_rej = step.id.clone();

        v_flex()
            .w_full()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                // Step header
                h_flex()
                    .w_full()
                    .px_3()
                    .py_2()
                    .bg(cx.theme().secondary.opacity(0.3))
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(IconName::Terminal).small().text_color(cx.theme().foreground))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(format!("Tool: {}", step.tool_name))
                            )
                    )
                    .child(
                        match step.status {
                            StepStatus::PendingApproval => Badge::new().child("Needs Approval"),
                            StepStatus::Running => Badge::new().child("Running..."),
                            StepStatus::Completed => Badge::new().child("Completed"),
                            StepStatus::Failed => Badge::new().child("Failed"),
                            StepStatus::Rejected => Badge::new().child("Rejected"),
                        }
                    )
            )
            // Command details / explanation
            .when(step.explanation.is_some(), |this| {
                this.child(
                    div()
                        .px_3()
                        .py_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(step.explanation.clone().unwrap())
                )
            })
            // Approval Banner (if waiting)
            .when(step.status == StepStatus::PendingApproval, |this| {
                this.child(
                    v_flex()
                        .p_3()
                        .bg(cx.theme().accent.opacity(0.15))
                        .border_t_1()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(cx.theme().foreground)
                                .child("This tool proposal requires authorization before running on your machine.")
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new(format!("app-{}", sid_app))
                                        .primary()
                                        .small()
                                        .icon(IconName::Check)
                                        .label("Approve & Run")
                                        .on_click(cx.listener({
                                            let on_approve_tool = on_approve_tool.clone();
                                            move |this, _, window, cx| on_approve_tool(this, sid_app.clone(), window, cx)
                                        }))
                                )
                                .child(
                                    Button::new(format!("rej-{}", sid_rej))
                                        .outline()
                                        .small()
                                        .icon(IconName::X)
                                        .label("Reject")
                                        .on_click(cx.listener({
                                            let on_reject_tool = on_reject_tool.clone();
                                            move |this, _, window, cx| on_reject_tool(this, sid_rej.clone(), window, cx)
                                        }))
                                )
                        )
                )
            })
            // Output console box
            .when(step.output.is_some(), |this| {
                this.child(
                    div()
                        .p_3()
                        .bg(cx.theme().secondary.opacity(0.4))
                        .max_h_64()
                        .overflow_hidden()
                        .text_xs()
                        .text_color(cx.theme().foreground)
                        .child(step.output.clone().unwrap())
                )
            })
    }
}
