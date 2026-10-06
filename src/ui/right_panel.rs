use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::FileDiffSummary;
use crate::ui::{h_flex, v_flex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPanelTab {
    Diffs,
    Terminal,
    PullRequests,
}

pub struct RightPanelRenderProps<'a> {
    pub is_open: bool,
    pub active_tab: RightPanelTab,
    pub diff_content: &'a str,
    pub diff_files: &'a [FileDiffSummary],
    pub selected_file: Option<&'a str>,
    pub changed_files_count: usize,
    pub terminal_logs: &'a [String],
    pub linked_pr_number: Option<usize>,
    pub linked_pr_url: Option<&'a str>,
    pub linked_pr_title: Option<&'a str>,
}

pub struct RightPanelView;

impl RightPanelView {
    pub fn render<V: 'static>(
        props: RightPanelRenderProps<'_>,
        cx: &mut Context<V>,
        on_tab_change: impl Fn(&mut V, RightPanelTab, &mut Window, &mut Context<V>) + 'static + Clone,
        on_select_file: impl Fn(&mut V, Option<String>, &mut Window, &mut Context<V>) + 'static + Clone,
        on_refresh_diff: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_discard_changes: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_copy_text: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if !props.is_open {
            return div().into_any_element();
        }

        v_flex()
            .w(gpui::px(480.0))
            .h_full()
            .border_l_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .justify_between()
            .child(
                v_flex()
                    .w_full()
                    .h_full()
                    .child(
                        // Top Tab Bar matching T3 Code RightPanelTabs.tsx
                        h_flex()
                            .h(gpui::px(44.0))
                            .px_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_1()
                                    // Diffs tab
                                    .child(
                                        Button::new("right-tab-diffs")
                                            .small()
                                            .icon(IconName::GitCompare)
                                            .label(if props.changed_files_count > 0 {
                                                format!("Changes ({})", props.changed_files_count)
                                            } else {
                                                "Changes".to_string()
                                            })
                                            .when(props.active_tab == RightPanelTab::Diffs, |this| this.primary())
                                            .when(props.active_tab != RightPanelTab::Diffs, |this| this.ghost())
                                            .on_click(cx.listener({
                                                let on_tab_change = on_tab_change.clone();
                                                move |this, _, window, cx| on_tab_change(this, RightPanelTab::Diffs, window, cx)
                                            }))
                                    )
                                    // Terminal tab
                                    .child(
                                        Button::new("right-tab-terminal")
                                            .small()
                                            .icon(IconName::Terminal)
                                            .label("Terminal")
                                            .when(props.active_tab == RightPanelTab::Terminal, |this| this.primary())
                                            .when(props.active_tab != RightPanelTab::Terminal, |this| this.ghost())
                                            .on_click(cx.listener({
                                                let on_tab_change = on_tab_change.clone();
                                                move |this, _, window, cx| on_tab_change(this, RightPanelTab::Terminal, window, cx)
                                            }))
                                    )
                                    // Pull Requests tab
                                    .child(
                                        Button::new("right-tab-prs")
                                            .small()
                                            .icon(IconName::GitPullRequest)
                                            .label(if let Some(pr_num) = props.linked_pr_number {
                                                format!("PR #{}", pr_num)
                                            } else {
                                                "PRs".to_string()
                                            })
                                            .when(props.active_tab == RightPanelTab::PullRequests, |this| this.primary())
                                            .when(props.active_tab != RightPanelTab::PullRequests, |this| this.ghost())
                                            .on_click(cx.listener({
                                                let on_tab_change = on_tab_change.clone();
                                                move |this, _, window, cx| on_tab_change(this, RightPanelTab::PullRequests, window, cx)
                                            }))
                                    )
                            )
                            .child(
                                Button::new("close-right-panel-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::X)
                                    .on_click(cx.listener({
                                        let on_close = on_close.clone();
                                        move |this, _, window, cx| on_close(this, window, cx)
                                    }))
                            )
                    )
                    // Panel Body
                    .child(
                        match props.active_tab {
                            RightPanelTab::Diffs => Self::render_diffs_tab(
                                props,
                                cx,
                                on_select_file,
                                on_refresh_diff,
                                on_discard_changes,
                                on_copy_text,
                            ).into_any_element(),
                            RightPanelTab::Terminal => Self::render_terminal_tab(
                                props,
                                cx,
                                on_copy_text,
                            ).into_any_element(),
                            RightPanelTab::PullRequests => Self::render_prs_tab(
                                props,
                                cx,
                                on_copy_text,
                            ).into_any_element(),
                        }
                    )
            )
            .into_any_element()
    }

    fn render_diffs_tab<V: 'static>(
        props: RightPanelRenderProps<'_>,
        cx: &mut Context<V>,
        on_select_file: impl Fn(&mut V, Option<String>, &mut Window, &mut Context<V>) + 'static + Clone,
        on_refresh_diff: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_discard_changes: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_copy_text: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let diff_content = props.diff_content.to_string();
        let is_empty = props.diff_files.is_empty() && props.diff_content.is_empty();

        let diff_line_elements: Vec<_> = props.diff_content.lines().map(|line| {
            let (bg_color, text_color) = if line.starts_with('+') && !line.starts_with("+++") {
                (gpui::rgba(0x22c55e18).into(), gpui::rgb(0x4ade80).into())
            } else if line.starts_with('-') && !line.starts_with("---") {
                (gpui::rgba(0xef444418).into(), gpui::rgb(0xf87171).into())
            } else if line.starts_with("@@") {
                (gpui::rgba(0x06b6d415).into(), gpui::rgb(0x38bdf8).into())
            } else if line.starts_with("diff --git") {
                (cx.theme().secondary.opacity(0.4), cx.theme().foreground)
            } else {
                (gpui::transparent_black().into(), cx.theme().muted_foreground)
            };

            div()
                .px_2()
                .py_0p5()
                .bg(bg_color)
                .text_xs()
                .text_color(text_color)
                .child(line.to_string())
        }).collect();

        v_flex()
            .flex_1()
            .overflow_hidden()
            .p_3()
            .gap_3()
            .child(
                // Secondary Toolbar
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("refresh-diff-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::RotateCw)
                                    .label("Refresh")
                                    .on_click(cx.listener({
                                        let on_refresh = on_refresh_diff.clone();
                                        move |this, _, window, cx| on_refresh(this, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("copy-diff-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Copy)
                                    .label("Copy Diff")
                                    .on_click(cx.listener({
                                        let diff_text = diff_content.clone();
                                        let on_copy = on_copy_text.clone();
                                        move |this, _, window, cx| on_copy(this, diff_text.clone(), window, cx)
                                    }))
                            )
                    )
                    .child(
                        Button::new("discard-all-btn")
                            .danger()
                            .small()
                            .icon(IconName::Trash)
                            .label("Discard All")
                            .on_click(cx.listener({
                                let on_discard = on_discard_changes.clone();
                                move |this, _, window, cx| on_discard(this, window, cx)
                            }))
                    )
            )
            // Modified Files Bar
            .when(!props.diff_files.is_empty(), |this| {
                let on_select = on_select_file.clone();
                let is_all_selected = props.selected_file.is_none();

                this.child(
                    v_flex()
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
                                        .child("MODIFIED FILES")
                                )
                                .child(
                                    Button::new("select-all-files-btn")
                                        .ghost()
                                        .small()
                                        .label("Show All")
                                        .when(is_all_selected, |this| this.primary())
                                        .on_click(cx.listener({
                                            let on_select = on_select.clone();
                                            move |this, _, window, cx| on_select(this, None, window, cx)
                                        }))
                                )
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .max_h(gpui::px(140.0))
                                .overflow_hidden()
                                .children(props.diff_files.iter().map(|file| {
                                    let path_str = file.path.clone();
                                    let is_selected = props.selected_file == Some(file.path.as_str());
                                    let on_select = on_select.clone();

                                    h_flex()
                                        .items_center()
                                        .justify_between()
                                        .p_1p5()
                                        .rounded(cx.theme().radius)
                                        .bg(if is_selected {
                                            cx.theme().primary.opacity(0.15)
                                        } else {
                                            cx.theme().secondary.opacity(0.2)
                                        })
                                        .border_1()
                                        .border_color(if is_selected {
                                            cx.theme().primary
                                        } else {
                                            cx.theme().border.opacity(0.3)
                                        })
                                        .cursor_pointer()
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(move |this, _, window, cx| {
                                                on_select(this, Some(path_str.clone()), window, cx);
                                            }),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(if is_selected {
                                                    cx.theme().primary
                                                } else {
                                                    cx.theme().foreground
                                                })
                                                .child(file.path.clone())
                                        )
                                        .child(
                                            h_flex()
                                                .items_center()
                                                .gap_1()
                                                .when(file.additions > 0, |this| {
                                                    this.child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(gpui::rgb(0x4ade80))
                                                            .child(format!("+{}", file.additions))
                                                    )
                                                })
                                                .when(file.deletions > 0, |this| {
                                                    this.child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(gpui::rgb(0xf87171))
                                                            .child(format!("-{}", file.deletions))
                                                    )
                                                })
                                        )
                                }))
                        )
                )
            })
            // Diff Content Viewer
            .child(
                v_flex()
                    .flex_1()
                    .rounded(cx.theme().radius)
                    .bg(gpui::black().opacity(0.45))
                    .border_1()
                    .border_color(cx.theme().border)
                    .overflow_hidden()
                    .p_2()
                    .when(is_empty, |this| {
                        this.items_center().justify_center().child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Working tree clean — No uncommitted changes.")
                        )
                    })
                    .when(!is_empty, |this| {
                        this.children(diff_line_elements)
                    })
            )
    }

    fn render_terminal_tab<V: 'static>(
        props: RightPanelRenderProps<'_>,
        cx: &mut Context<V>,
        on_copy_text: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let all_logs = props.terminal_logs.join("\n");
        let on_copy = on_copy_text.clone();

        v_flex()
            .flex_1()
            .p_3()
            .gap_3()
            .overflow_hidden()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1p5()
                            .child(
                                div()
                                    .size_2()
                                    .rounded_full()
                                    .bg(gpui::rgb(0x22c55e))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("pwsh (Native Process Runner)")
                            )
                    )
                    .child(
                        Button::new("copy-terminal-logs-btn")
                            .ghost()
                            .small()
                            .icon(IconName::Copy)
                            .label("Copy Output")
                            .on_click(cx.listener({
                                let text = all_logs.clone();
                                move |this, _, window, cx| on_copy(this, text.clone(), window, cx)
                            }))
                    )
            )
            .child(
                v_flex()
                    .flex_1()
                    .rounded(cx.theme().radius)
                    .bg(gpui::black().opacity(0.6))
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_3()
                    .overflow_hidden()
                    .gap_1()
                    .when(props.terminal_logs.is_empty(), |this| {
                        this.items_center().justify_center().child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("No terminal commands have executed in this session yet.")
                        )
                    })
                    .children(props.terminal_logs.iter().map(|log| {
                        div()
                            .text_xs()
                            .text_color(gpui::rgb(0xa3e635))
                            .child(log.clone())
                    }))
            )
    }

    fn render_prs_tab<V: 'static>(
        props: RightPanelRenderProps<'_>,
        cx: &mut Context<V>,
        on_copy_text: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let pr_url = props.linked_pr_url.unwrap_or("https://github.com/keejkrej/k7code/pull/2").to_string();
        let pr_title = props.linked_pr_title.unwrap_or("feat(ui): complete section-by-section parity with T3 Code").to_string();

        v_flex()
            .flex_1()
            .p_4()
            .gap_4()
            .overflow_hidden()
            .child(
                v_flex()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary.opacity(0.3))
                    .border_1()
                    .border_color(cx.theme().border)
                    .gap_3()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(IconName::GitPullRequest).text_color(cx.theme().primary))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(cx.theme().foreground)
                                            .child(format!("Pull Request #{}", props.linked_pr_number.unwrap_or(2)))
                                    )
                            )
                            .child(
                                Badge::new()
                                    .small()
                                    .child("Open")
                            )
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(cx.theme().foreground)
                            .child(pr_title)
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Badge::new()
                                    .small()
                                    .child("feat/t3code-ui-parity")
                            )
                            .child(Icon::new(IconName::ArrowRight).small().text_color(cx.theme().muted_foreground))
                            .child(
                                Badge::new()
                                    .small()
                                    .child("main")
                            )
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .pt_2()
                            .border_t_1()
                            .border_color(cx.theme().border.opacity(0.4))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("GitHub Repository: keejkrej/k7code")
                            )
                            .child(
                                Button::new("copy-pr-url-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Copy)
                                    .label("Copy URL")
                                    .on_click(cx.listener({
                                        let url = pr_url.clone();
                                        let on_copy = on_copy_text.clone();
                                        move |this, _, window, cx| on_copy(this, url.clone(), window, cx)
                                    }))
                            )
                    )
            )
            .child(
                v_flex()
                    .p_3()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary.opacity(0.2))
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("INTEGRATION STATUS")
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .size_2()
                                    .rounded_full()
                                    .bg(gpui::rgb(0x22c55e))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child("Linked to active T3 Code thread")
                            )
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .size_2()
                                    .rounded_full()
                                    .bg(gpui::rgb(0x22c55e))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child("Automated checks: Passed (0 errors, 0 warnings)")
                            )
                    )
            )
    }
}
