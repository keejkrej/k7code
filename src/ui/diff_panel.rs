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

pub struct DiffPanelRenderProps<'a> {
    pub files: &'a [FileDiffSummary],
    pub diff_content: &'a str,
    pub selected_file: Option<&'a str>,
    pub is_open: bool,
}

pub struct DiffPanelView;

impl DiffPanelView {
    pub fn render<V: 'static>(
        props: DiffPanelRenderProps<'_>,
        cx: &mut Context<V>,
        on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_refresh: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_select_file: impl Fn(&mut V, Option<String>, &mut Window, &mut Context<V>) + 'static + Clone,
        on_copy_diff: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_revert_all: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if !props.is_open {
            return div().into_any_element();
        }

        // Parse diff lines into syntax colored elements
        let mut diff_line_elements = Vec::new();
        if props.diff_content.is_empty() {
            diff_line_elements.push(
                div()
                    .p_4()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Working tree clean. No local modifications.")
                    .into_any_element(),
            );
        } else {
            for line in props.diff_content.lines().take(400) {
                if line.starts_with('+') && !line.starts_with("+++") {
                    diff_line_elements.push(
                        div()
                            .w_full()
                            .px_2()
                            .py_0p5()
                            .bg(gpui::rgba(0x22c55e1a))
                            .text_color(gpui::rgb(0x4ade80))
                            .text_xs()
                            .child(line.to_string())
                            .into_any_element(),
                    );
                } else if line.starts_with('-') && !line.starts_with("---") {
                    diff_line_elements.push(
                        div()
                            .w_full()
                            .px_2()
                            .py_0p5()
                            .bg(gpui::rgba(0xef44441a))
                            .text_color(gpui::rgb(0xf87171))
                            .text_xs()
                            .child(line.to_string())
                            .into_any_element(),
                    );
                } else if line.starts_with("@@") {
                    diff_line_elements.push(
                        div()
                            .w_full()
                            .px_2()
                            .py_0p5()
                            .bg(gpui::rgba(0x38bdf81a))
                            .text_color(gpui::rgb(0x38bdf8))
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(line.to_string())
                            .into_any_element(),
                    );
                } else {
                    diff_line_elements.push(
                        div()
                            .w_full()
                            .px_2()
                            .py_0p5()
                            .text_color(cx.theme().muted_foreground)
                            .text_xs()
                            .child(line.to_string())
                            .into_any_element(),
                    );
                }
            }
        }

        let diff_content_string = props.diff_content.to_string();

        v_flex()
            .w(gpui::px(440.0))
            .h_full()
            .border_l_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .justify_between()
            .child(
                v_flex()
                    .w_full()
                    .child(
                        // Header
                        h_flex()
                            .h(gpui::px(48.0))
                            .px_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(IconName::GitCompare).small().text_color(cx.theme().primary))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().foreground)
                                            .child("Working Tree Changes")
                                    )
                                    .child(
                                        Badge::new()
                                            .small()
                                            .child(format!("{} files", props.files.len()))
                                    )
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        Button::new("refresh-diff-btn")
                                            .ghost()
                                            .small()
                                            .icon(IconName::RotateCw)
                                            .on_click(cx.listener({
                                                let on_refresh = on_refresh.clone();
                                                move |this, _, window, cx| on_refresh(this, window, cx)
                                            }))
                                    )
                                    .child(
                                        Button::new("close-diff-btn")
                                            .ghost()
                                            .small()
                                            .icon(IconName::X)
                                            .on_click(cx.listener({
                                                let on_close = on_close.clone();
                                                move |this, _, window, cx| on_close(this, window, cx)
                                            }))
                                    )
                            )
                    )
                    // Modified files list
                    .child(
                        v_flex()
                            .p_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .gap_1p5()
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
                                        Button::new("all-files-diff-btn")
                                            .ghost()
                                            .small()
                                            .label("Show All")
                                            .when(props.selected_file.is_none(), |this| this.primary())
                                            .on_click(cx.listener({
                                                let on_select_file = on_select_file.clone();
                                                move |this, _, window, cx| on_select_file(this, None, window, cx)
                                            }))
                                    )
                            )
                            .when(props.files.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Working directory clean")
                                )
                            })
                            .children(props.files.iter().map(|f| {
                                let file_path = f.path.clone();
                                let is_selected = props.selected_file == Some(&file_path);
                                let on_select_file = on_select_file.clone();
                                let file_path_click = file_path.clone();

                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .justify_between()
                                    .px_2()
                                    .py_1()
                                    .rounded(cx.theme().radius)
                                    .border_1()
                                    .border_color(if is_selected {
                                        cx.theme().primary
                                    } else {
                                        gpui::transparent_black()
                                    })
                                    .bg(if is_selected {
                                        cx.theme().primary.opacity(0.12)
                                    } else {
                                        cx.theme().secondary.opacity(0.2)
                                    })
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, window, cx| {
                                            on_select_file(this, Some(file_path_click.clone()), window, cx);
                                        }),
                                    )
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_1p5()
                                            .child(Icon::new(IconName::FileCode).small().text_color(cx.theme().muted_foreground))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(if is_selected {
                                                        cx.theme().primary
                                                    } else {
                                                        cx.theme().foreground
                                                    })
                                                    .child(file_path)
                                            )
                                    )
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_1()
                                            .when(f.additions > 0, |this| {
                                                this.child(
                                                    Badge::new()
                                                        .small()
                                                        .child(format!("+{}", f.additions))
                                                )
                                            })
                                            .when(f.deletions > 0, |this| {
                                                this.child(
                                                    Badge::new()
                                                        .small()
                                                        .child(format!("-{}", f.deletions))
                                                )
                                            })
                                    )
                            }))
                    )
                    // Unified Diff Content
                    .child(
                        v_flex()
                            .p_3()
                            .gap_2()
                            .child(
                                h_flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().muted_foreground)
                                            .child(if let Some(file) = props.selected_file {
                                                format!("DIFF: {file}")
                                            } else {
                                                "UNIFIED DIFF".to_string()
                                            })
                                    )
                                    .child(
                                        Button::new("copy-unified-diff-btn")
                                            .ghost()
                                            .small()
                                            .icon(IconName::Copy)
                                            .label("Copy Diff")
                                            .on_click(cx.listener({
                                                let diff_str = diff_content_string.clone();
                                                let on_copy_diff = on_copy_diff.clone();
                                                move |this, _, window, cx| on_copy_diff(this, diff_str.clone(), window, cx)
                                            }))
                                    )
                            )
                            .child(
                                v_flex()
                                    .rounded(cx.theme().radius)
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .bg(cx.theme().secondary.opacity(0.2))
                                    .max_h(gpui::px(420.0))
                                    .overflow_hidden()
                                    .children(diff_line_elements)
                            )
                    )
            )
            // Footer Revert Button
            .child(
                v_flex()
                    .p_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        Button::new("revert-all-btn")
                            .danger()
                            .w_full()
                            .icon(IconName::RotateCcw)
                            .label("Discard All Changes (Hard Reset)")
                            .on_click(cx.listener({
                                let on_revert_all = on_revert_all.clone();
                                move |this, _, window, cx| on_revert_all(this, window, cx)
                            }))
                    )
            )
            .into_any_element()
    }
}
