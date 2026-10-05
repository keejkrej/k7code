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
    pub is_open: bool,
}

pub struct DiffPanelView;

impl DiffPanelView {
    pub fn render<V: 'static>(
        props: DiffPanelRenderProps<'_>,
        cx: &mut Context<V>,
        on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_revert_all: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if !props.is_open {
            return div().into_any_element();
        }

        v_flex()
            .w_96()
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
                            .h(gpui::px(56.0))
                            .px_4()
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
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().foreground)
                                            .child("Thread Diff & Changes")
                                    )
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
                    // Modified files list
                    .child(
                        v_flex()
                            .p_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("CHANGED FILES ({})", props.files.len()))
                            )
                            .when(props.files.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("No modified files in working tree.")
                                )
                            })
                            .children(props.files.iter().map(|f| {
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .justify_between()
                                    .py_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().foreground)
                                            .child(f.path.clone())
                                    )
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_1()
                                            .when(f.additions > 0, |this| {
                                                this.child(
                                                    Badge::new()
                                                        .child(format!("+{}", f.additions))
                                                )
                                            })
                                            .when(f.deletions > 0, |this| {
                                                this.child(
                                                    Badge::new()
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
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("UNIFIED DIFF")
                            )
                            .child(
                                div()
                                    .p_3()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().secondary.opacity(0.2))
                                    .max_h_96()
                                    .overflow_hidden()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child(if props.diff_content.is_empty() {
                                        "No git diff available.".to_string()
                                    } else {
                                        props.diff_content.to_string()
                                    })
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
                            .label("Discard All Uncommitted Changes")
                            .on_click(cx.listener({
                                let on_revert_all = on_revert_all.clone();
                                move |this, _, window, cx| on_revert_all(this, window, cx)
                            }))
                    )
            )
            .into_any_element()
    }
}
