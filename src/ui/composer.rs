use gpui_kit::assets::IconName;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    ActiveTheme, Sizable,
};
use gpui_kit::*;

use crate::model::ProviderKind;
use crate::ui::{h_flex, v_flex};

pub struct ComposerRenderProps<'a> {
    pub input_state: &'a Entity<InputState>,
    pub provider: ProviderKind,
    pub model: &'a str,
    pub is_running: bool,
}

pub struct ComposerView;

impl ComposerView {
    pub fn render<V: 'static>(
        props: ComposerRenderProps<'_>,
        cx: &mut Context<V>,
        on_submit: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_stop: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_cycle_model: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
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
                    // Bottom Controls Bar
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                // Model & Provider Selector Pill
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Button::new("composer-model-pill")
                                            .secondary()
                                            .small()
                                            .icon(IconName::Sparkles)
                                            .label(format!("{} · {}", props.provider.display_name(), props.model))
                                            .on_click(cx.listener({
                                                let on_cycle_model = on_cycle_model.clone();
                                                move |this, _, window, cx| on_cycle_model(this, window, cx)
                                            }))
                                    )
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
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
            .child(
                div()
                    .pt_1()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground.opacity(0.8))
                    .child("Enter to send · Shift+Enter for new line · Click model pill to switch CLI provider")
            )
    }
}
