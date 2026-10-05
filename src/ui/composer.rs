use gpui_kit::assets::IconName;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{ProviderKind, RuntimeMode};
use crate::ui::{h_flex, v_flex};

pub struct ComposerRenderProps<'a> {
    pub input_state: &'a Entity<InputState>,
    pub provider: ProviderKind,
    pub model: &'a str,
    pub runtime_mode: RuntimeMode,
    pub project_name: &'a str,
    pub git_branch: Option<&'a str>,
    pub is_running: bool,
    pub used_tokens: usize,
    pub max_tokens: usize,
}

pub struct ComposerView;

impl ComposerView {
    pub fn render<V: 'static>(
        props: ComposerRenderProps<'_>,
        cx: &mut Context<V>,
        on_submit: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_stop: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_open_model_picker: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_clear_prompt: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let percent = if props.max_tokens > 0 {
            (props.used_tokens * 100) / props.max_tokens
        } else {
            0
        };

        // Floating glass composer surface matching T3 Code ComposerSurface.tsx
        v_flex()
            .w_full()
            .items_center()
            .px_6()
            .pb_4()
            .pt_2()
            .child(
                v_flex()
                    .w_full()
                    .max_w(gpui::px(820.0))
                    // Upper Main Composer Container (rounded-[22px], elevated, subtle highlight)
                    .child(
                        v_flex()
                            .w_full()
                            .rounded(cx.theme().radius)
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().secondary.opacity(0.35))
                            .p_3()
                            .gap_2p5()
                            // Input prompt area
                            .child(
                                Input::new(props.input_state)
                                    .bordered(false)
                            )
                            // Bottom Controls Bar inside Main Surface
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        // Left: Model & Provider Selector Pill + Context Mention Trigger
                                        h_flex()
                                            .items_center()
                                            .gap_1p5()
                                            .child(
                                                Button::new("composer-model-pill")
                                                    .secondary()
                                                    .small()
                                                    .icon(IconName::Sparkles)
                                                    .label(format!("{} · {}", props.provider.display_name(), props.model))
                                                    .on_click(cx.listener({
                                                        let on_open_model_picker = on_open_model_picker.clone();
                                                        move |this, _, window, cx| on_open_model_picker(this, window, cx)
                                                    }))
                                            )
                                            .child(
                                                Button::new("composer-context-mention-btn")
                                                    .ghost()
                                                    .small()
                                                    .label("@ Context")
                                                    .on_click(cx.listener(|_, _, _, _| {
                                                        // Context reference mention trigger
                                                    }))
                                            )
                                    )
                                    .child(
                                        // Right: Context Window Token Meter + Clear + Send/Stop
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            // Context Window Meter matching ContextWindowMeter.tsx
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded_full()
                                                    .bg(cx.theme().secondary.opacity(0.4))
                                                    .border_1()
                                                    .border_color(cx.theme().border.opacity(0.5))
                                                    .child(
                                                        div()
                                                            .size_2()
                                                            .rounded_full()
                                                            .bg(if percent > 85 {
                                                                gpui::rgb(0xf87171)
                                                            } else {
                                                                gpui::rgb(0x38bdf8)
                                                            })
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(format!("{:.1}k / {}k · {}%", props.used_tokens as f32 / 1000.0, props.max_tokens / 1000, percent))
                                                    )
                                            )
                                            .child(
                                                Button::new("composer-clear-btn")
                                                    .ghost()
                                                    .small()
                                                    .icon(IconName::X)
                                                    .label("Clear")
                                                    .on_click(cx.listener({
                                                        let on_clear_prompt = on_clear_prompt.clone();
                                                        move |this, _, window, cx| on_clear_prompt(this, window, cx)
                                                    }))
                                            )
                                            .child(
                                                if props.is_running {
                                                    // Stop execution button
                                                    Button::new("composer-stop-btn")
                                                        .danger()
                                                        .small()
                                                        .icon(IconName::Square)
                                                        .label("Stop")
                                                        .on_click(cx.listener({
                                                            let on_stop = on_stop.clone();
                                                            move |this, _, window, cx| on_stop(this, window, cx)
                                                        }))
                                                } else {
                                                    // Send / Run button
                                                    Button::new("composer-send-btn")
                                                        .primary()
                                                        .small()
                                                        .icon(IconName::ArrowUp)
                                                        .label("Send")
                                                        .on_click(cx.listener({
                                                            let on_submit = on_submit.clone();
                                                            move |this, _, window, cx| on_submit(this, window, cx)
                                                        }))
                                                }
                                            )
                                    )
                            )
                    )
                    // Attached Context Strip underneath the surface matching T3 Code ContextStrip
                    .child(
                        h_flex()
                            .mx_4()
                            .px_3()
                            .py_1()
                            .rounded_b(cx.theme().radius)
                            .bg(cx.theme().secondary.opacity(0.2))
                            .border_l_1()
                            .border_r_1()
                            .border_b_1()
                            .border_color(cx.theme().border.opacity(0.4))
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_1()
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
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("Local Machine")
                                            )
                                    )
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(0.7))
                                    .child(format!("Mode: {}", props.runtime_mode.label()))
                            )
                    )
            )
            .child(
                div()
                    .pt_1()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground.opacity(0.8))
                    .child("Enter to send · Shift+Enter newline · Esc to clear · Ctrl+B sidebar · Ctrl+D diffs · Ctrl+` terminal")
            )
    }
}
