use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{Project, RuntimeMode, Thread};
use crate::ui::h_flex;

pub struct HeaderRenderProps<'a> {
    pub active_project: Option<&'a Project>,
    pub active_thread: Option<&'a Thread>,
    pub git_branch: Option<&'a str>,
    pub changed_files_count: usize,
    pub is_diff_panel_open: bool,
}

pub struct HeaderView;

impl HeaderView {
    pub fn render<V: 'static>(
        props: HeaderRenderProps<'_>,
        cx: &mut Context<V>,
        on_toggle_diff: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_change_runtime_mode: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_change_approval_policy: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let project_name = props
            .active_project
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Workspace".to_string());

        let (title, is_running, runtime_mode, approval_policy) = match props.active_thread {
            Some(t) => (
                t.title.clone(),
                t.is_running(),
                t.runtime_mode,
                t.approval_policy.label(),
            ),
            None => (
                "New Thread".to_string(),
                false,
                RuntimeMode::AutoAcceptEdits,
                "Untrusted",
            ),
        };

        let branch_name = props.git_branch.unwrap_or("main");

        let (mode_icon, mode_label) = match runtime_mode {
            RuntimeMode::Supervised => (IconName::Lock, "Supervised"),
            RuntimeMode::AutoAcceptEdits => (IconName::PenLine, "Auto-accept edits"),
            RuntimeMode::Auto => (IconName::Sparkles, "Auto"),
            RuntimeMode::FullAccess => (IconName::LockOpen, "Full access"),
        };

        h_flex()
            .h(gpui::px(52.0))
            .w_full()
            .px_4()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .items_center()
            .justify_between()
            .child(
                // Left side: Workspace Breadcrumb: [Project] > [Thread Title]
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1p5()
                            .child(Icon::new(IconName::Folder).small().text_color(cx.theme().muted_foreground))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().muted_foreground)
                                    .child(project_name)
                            )
                    )
                    .child(
                        Icon::new(IconName::ChevronRight)
                            .small()
                            .text_color(cx.theme().muted_foreground.opacity(0.6))
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child(title)
                    )
                    .when(is_running, |this| {
                        this.child(
                            Badge::new()
                                .small()
                                .child("Thinking...")
                        )
                    })
            )
            .child(
                // Right side: Git Branch, Runtime Mode, Approval Policy, Diff Drawer
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        // Git Branch pill
                        h_flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_1()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().secondary.opacity(0.4))
                            .child(Icon::new(IconName::GitBranch).small().text_color(cx.theme().muted_foreground))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().foreground)
                                    .child(branch_name.to_string())
                            )
                    )
                    .child(
                        // T3 Code Runtime Mode button
                        Button::new("runtime-mode-btn")
                            .secondary()
                            .small()
                            .icon(mode_icon)
                            .label(mode_label)
                            .on_click(cx.listener({
                                let on_change_runtime_mode = on_change_runtime_mode.clone();
                                move |this, _, window, cx| on_change_runtime_mode(this, window, cx)
                            }))
                    )
                    .child(
                        // Approval Policy button
                        Button::new("approval-policy-btn")
                            .ghost()
                            .small()
                            .icon(IconName::CheckCheck)
                            .label(approval_policy)
                            .on_click(cx.listener({
                                let on_change_approval_policy = on_change_approval_policy.clone();
                                move |this, _, window, cx| on_change_approval_policy(this, window, cx)
                            }))
                    )
                    .child(
                        // Diff Drawer Toggle
                        Button::new("diff-toggle-btn")
                            .when(props.is_diff_panel_open, |btn| btn.primary())
                            .when(!props.is_diff_panel_open, |btn| btn.outline())
                            .small()
                            .icon(IconName::GitCompare)
                            .label(if props.changed_files_count > 0 {
                                format!("Diff ({})", props.changed_files_count)
                            } else {
                                "Diff".to_string()
                            })
                            .on_click(cx.listener({
                                let on_toggle_diff = on_toggle_diff.clone();
                                move |this, _, window, cx| on_toggle_diff(this, window, cx)
                            }))
                    )
            )
    }
}
