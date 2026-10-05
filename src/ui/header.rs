use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{ApprovalPolicy, RuntimeMode};
use crate::ui::h_flex;

pub struct HeaderRenderProps<'a> {
    pub project_name: &'a str,
    pub git_branch: Option<&'a str>,
    pub thread_title: &'a str,
    pub runtime_mode: RuntimeMode,
    pub approval_policy: ApprovalPolicy,
    pub is_diff_open: bool,
    pub changed_files_count: usize,
    pub is_running: bool,
    pub is_sidebar_open: bool,
}

pub struct HeaderView;

impl HeaderView {
    pub fn render<V: 'static>(
        props: HeaderRenderProps<'_>,
        cx: &mut Context<V>,
        on_toggle_sidebar: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_diff: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_cycle_runtime_mode: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_cycle_approval_policy: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_open_rename: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_new_thread: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_open_settings: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let mode_icon = match props.runtime_mode {
            RuntimeMode::Supervised => IconName::Lock,
            RuntimeMode::AutoAcceptEdits => IconName::PenLine,
            RuntimeMode::Auto => IconName::Sparkles,
            RuntimeMode::FullAccess => IconName::LockOpen,
        };

        h_flex()
            .h(gpui::px(48.0))
            .w_full()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .px_3()
            .items_center()
            .justify_between()
            .child(
                // Left Navigation: Sidebar Toggle + Breadcrumb
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("toggle-sidebar-btn")
                            .ghost()
                            .small()
                            .icon(if props.is_sidebar_open {
                                IconName::PanelLeftClose
                            } else {
                                IconName::PanelLeft
                            })
                            .on_click(cx.listener({
                                let on_toggle_sidebar = on_toggle_sidebar.clone();
                                move |this, _, window, cx| on_toggle_sidebar(this, window, cx)
                            }))
                    )
                    .child(
                        // Project Monogram & Name
                        h_flex()
                            .items_center()
                            .gap_1p5()
                            .px_2()
                            .py_1()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().secondary.opacity(0.4))
                            .child(Icon::new(IconName::Folder).small().text_color(cx.theme().primary))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(props.project_name.to_string())
                            )
                    )
                    .child(
                        Icon::new(IconName::ChevronRight)
                            .small()
                            .text_color(cx.theme().muted_foreground.opacity(0.6))
                    )
                    .when_some(props.git_branch, |this, branch| {
                        this.child(
                            h_flex()
                                .items_center()
                                .gap_1()
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .border_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().secondary.opacity(0.2))
                                .child(Icon::new(IconName::GitBranch).small().text_color(cx.theme().muted_foreground))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(branch.to_string())
                                )
                        )
                        .child(
                            Icon::new(IconName::ChevronRight)
                                .small()
                                .text_color(cx.theme().muted_foreground.opacity(0.6))
                        )
                    })
                    // Active Thread Title with Rename Action
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().foreground)
                                    .child(props.thread_title.to_string())
                            )
                            .child(
                                Button::new("rename-thread-header-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Pencil)
                                    .on_click(cx.listener({
                                        let on_open_rename = on_open_rename.clone();
                                        move |this, _, window, cx| on_open_rename(this, window, cx)
                                    }))
                            )
                    )
                    .when(props.is_running, |this| {
                        this.child(
                            Badge::new()
                                .small()
                                .child("Thinking...")
                        )
                    })
            )
            .child(
                // Right Controls Bar
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        // New Thread Quick Button
                        Button::new("header-new-thread-btn")
                            .ghost()
                            .small()
                            .icon(IconName::Plus)
                            .label("New Thread")
                            .on_click(cx.listener({
                                let on_new_thread = on_new_thread.clone();
                                move |this, _, window, cx| on_new_thread(this, window, cx)
                            }))
                    )
                    .child(
                        // Runtime Mode Switcher
                        Button::new("runtime-mode-btn")
                            .secondary()
                            .small()
                            .icon(mode_icon)
                            .label(props.runtime_mode.label())
                            .on_click(cx.listener({
                                let on_cycle_runtime_mode = on_cycle_runtime_mode.clone();
                                move |this, _, window, cx| on_cycle_runtime_mode(this, window, cx)
                            }))
                    )
                    .child(
                        // Approval Policy Switcher
                        Button::new("approval-policy-btn")
                            .ghost()
                            .small()
                            .icon(IconName::ShieldCheck)
                            .label(props.approval_policy.label())
                            .on_click(cx.listener({
                                let on_cycle_approval_policy = on_cycle_approval_policy.clone();
                                move |this, _, window, cx| on_cycle_approval_policy(this, window, cx)
                            }))
                    )
                    .child(
                        // Diff Panel Drawer Button with Changes Count
                        Button::new("diff-toggle-btn")
                            .small()
                            .icon(IconName::GitCompare)
                            .label(if props.changed_files_count > 0 {
                                format!("Diff ({})", props.changed_files_count)
                            } else {
                                "Diff".to_string()
                            })
                            .when(props.is_diff_open, |this| this.primary())
                            .when(!props.is_diff_open, |this| this.secondary())
                            .on_click(cx.listener({
                                let on_toggle_diff = on_toggle_diff.clone();
                                move |this, _, window, cx| on_toggle_diff(this, window, cx)
                            }))
                    )
                    .child(
                        // Settings Gear Button
                        Button::new("header-settings-btn")
                            .ghost()
                            .small()
                            .icon(IconName::Settings)
                            .on_click(cx.listener({
                                let on_open_settings = on_open_settings.clone();
                                move |this, _, window, cx| on_open_settings(this, window, cx)
                            }))
                    )
            )
    }
}
