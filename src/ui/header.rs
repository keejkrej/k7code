use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{ApprovalPolicy, RuntimeMode};
use crate::ui::right_panel::RightPanelTab;
use crate::ui::h_flex;

pub struct HeaderRenderProps<'a> {
    pub is_sidebar_open: bool,
    pub is_right_panel_open: bool,
    pub active_right_tab: RightPanelTab,
    pub project_name: &'a str,
    pub git_branch: Option<&'a str>,
    pub thread_title: Option<&'a str>,
    pub model: &'a str,
    pub runtime_mode: RuntimeMode,
    pub approval_policy: ApprovalPolicy,
    pub changed_files_count: usize,
    pub linked_pr_number: Option<usize>,
}

pub struct HeaderView;

impl HeaderView {
    pub fn render<V: 'static>(
        props: HeaderRenderProps<'_>,
        cx: &mut Context<V>,
        on_toggle_sidebar: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_right_tab: impl Fn(&mut V, RightPanelTab, &mut Window, &mut Context<V>) + 'static + Clone,
        on_new_thread: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_rename_thread: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_runtime_mode: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_approval_policy: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_open_settings: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let branch_name = props.git_branch.unwrap_or("no-git").to_string();
        let thread_label = props.thread_title.unwrap_or("New Conversation").to_string();

        h_flex()
            .h(gpui::px(48.0))
            .w_full()
            .px_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .items_center()
            .justify_between()
            .child(
                // Left Navigation Section: Sidebar Toggle + Breadcrumb
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
                    // Project Monogram & Name
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1p5()
                            .px_2()
                            .py_0p5()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().secondary.opacity(0.4))
                            .child(
                                div()
                                    .size_4()
                                    .rounded_full()
                                    .bg(cx.theme().primary)
                                    .text_color(gpui::white())
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(props.project_name.chars().next().unwrap_or('P').to_string().to_uppercase())
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(props.project_name.to_string())
                            )
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("/")
                    )
                    // Branch Selector Badge
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1()
                            .px_2()
                            .py_0p5()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().secondary.opacity(0.3))
                            .child(Icon::new(IconName::GitBranch).small().text_color(cx.theme().muted_foreground))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().foreground)
                                    .child(branch_name)
                            )
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("/")
                    )
                    // Thread Title with Inline Edit Pencil
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().foreground)
                                    .child(thread_label)
                            )
                            .child(
                                Button::new("header-rename-thread-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Pencil)
                                    .on_click(cx.listener({
                                        let on_rename = on_rename_thread.clone();
                                        move |this, _, window, cx| on_rename(this, window, cx)
                                    }))
                            )
                    )
            )
            .child(
                // Right Action Section: Mode badges + Right Panel Tabs + New Thread + Settings
                h_flex()
                    .items_center()
                    .gap_1p5()
                    // Environment Mode: Local
                    .child(
                        Badge::new()
                            .small()
                            .child("Local")
                    )
                    // Runtime Mode Button
                    .child(
                        Button::new("header-mode-toggle-btn")
                            .ghost()
                            .small()
                            .label(props.runtime_mode.label())
                            .on_click(cx.listener({
                                let on_toggle = on_toggle_runtime_mode.clone();
                                move |this, _, window, cx| on_toggle(this, window, cx)
                            }))
                    )
                    // Approval Policy Button
                    .child(
                        Button::new("header-policy-toggle-btn")
                            .ghost()
                            .small()
                            .label(props.approval_policy.label())
                            .on_click(cx.listener({
                                let on_toggle = on_toggle_approval_policy.clone();
                                move |this, _, window, cx| on_toggle(this, window, cx)
                            }))
                    )
                    // Changes / Diffs Drawer Toggle
                    .child({
                        let is_diff_open = props.is_right_panel_open && props.active_right_tab == RightPanelTab::Diffs;
                        let on_tab = on_toggle_right_tab.clone();

                        Button::new("header-diff-toggle-btn")
                            .small()
                            .icon(IconName::GitCompare)
                            .label(if props.changed_files_count > 0 {
                                format!("Diff ({})", props.changed_files_count)
                            } else {
                                "Diff".to_string()
                            })
                            .when(is_diff_open, |this| this.primary())
                            .when(!is_diff_open, |this| this.secondary())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                on_tab(this, RightPanelTab::Diffs, window, cx);
                            }))
                    })
                    // Terminal Drawer Toggle
                    .child({
                        let is_term_open = props.is_right_panel_open && props.active_right_tab == RightPanelTab::Terminal;
                        let on_tab = on_toggle_right_tab.clone();

                        Button::new("header-term-toggle-btn")
                            .small()
                            .icon(IconName::Terminal)
                            .label("Terminal")
                            .when(is_term_open, |this| this.primary())
                            .when(!is_term_open, |this| this.secondary())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                on_tab(this, RightPanelTab::Terminal, window, cx);
                            }))
                    })
                    // Pull Request Drawer Toggle
                    .child({
                        let is_pr_open = props.is_right_panel_open && props.active_right_tab == RightPanelTab::PullRequests;
                        let on_tab = on_toggle_right_tab.clone();

                        Button::new("header-pr-toggle-btn")
                            .small()
                            .icon(IconName::GitPullRequest)
                            .label(if let Some(pr_num) = props.linked_pr_number {
                                format!("PR #{}", pr_num)
                            } else {
                                "PRs".to_string()
                            })
                            .when(is_pr_open, |this| this.primary())
                            .when(!is_pr_open, |this| this.secondary())
                            .on_click(cx.listener(move |this, _, window, cx| {
                                on_tab(this, RightPanelTab::PullRequests, window, cx);
                            }))
                    })
                    // New Thread Action Button
                    .child(
                        Button::new("header-new-thread-btn")
                            .primary()
                            .small()
                            .icon(IconName::Plus)
                            .on_click(cx.listener({
                                let on_new_thread = on_new_thread.clone();
                                move |this, _, window, cx| on_new_thread(this, window, cx)
                            }))
                    )
                    // Settings Button
                    .child(
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
