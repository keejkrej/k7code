use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::Thread;
use crate::ui::h_flex;

pub struct HeaderRenderProps<'a> {
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
        let (title, is_running, runtime_mode, approval_policy) = match props.active_thread {
            Some(t) => (
                t.title.clone(),
                t.is_running(),
                t.runtime_mode.label(),
                t.approval_policy.label(),
            ),
            None => (
                "No Thread Selected".to_string(),
                false,
                "Workspace Write",
                "On Request",
            ),
        };

        let branch_name = props.git_branch.unwrap_or("detached");

        h_flex()
            .h(gpui::px(56.0))
            .w_full()
            .px_4()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .items_center()
            .justify_between()
            .child(
                // Left side: Thread Title & Status indicator
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(title)
                            )
                            .when(is_running, |this| {
                                this.child(
                                    Badge::new()
                                        .child("Thinking...")
                                )
                            })
                    )
            )
            .child(
                // Right side: Git Branch, Mode, Approval Policy, Diff Toggle
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        // Git Branch pill
                        h_flex()
                            .items_center()
                            .gap_1()
                            .px_2()
                            .py_1()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().secondary.opacity(0.4))
                            .child(Icon::new(IconName::GitBranch).small().text_color(cx.theme().muted_foreground))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child(branch_name.to_string())
                            )
                    )
                    .child(
                        // Runtime Mode button
                        Button::new("runtime-mode-btn")
                            .ghost()
                            .small()
                            .icon(IconName::ShieldAlert)
                            .label(format!("Mode: {}", runtime_mode))
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
                            .label(format!("Approval: {}", approval_policy))
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
