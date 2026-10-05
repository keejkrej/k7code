use gpui_kit::assets::IconName;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    Sizable, ActiveTheme,
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
        v_flex()
            .w_full()
            .p_4()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .gap_2()
            .child(
                // Input bar container
                v_flex()
                    .w_full()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().secondary.opacity(0.15))
                    .p_2()
                    .gap_2()
                    .child(
                        Input::new(props.input_state)
                            .bordered(false)
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                // Model & Provider switcher button
                                Button::new("model-select-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Cpu)
                                    .label(format!("{}: {}", props.provider.display_name(), props.model))
                                    .on_click(cx.listener({
                                        let on_cycle_model = on_cycle_model.clone();
                                        move |this, _, window, cx| on_cycle_model(this, window, cx)
                                    }))
                            )
                            .child(
                                if props.is_running {
                                    // Stop / Cancel Button
                                    Button::new("stop-btn")
                                        .danger()
                                        .small()
                                        .icon(IconName::Square)
                                        .label("Stop")
                                        .on_click(cx.listener({
                                            let on_stop = on_stop.clone();
                                            move |this, _, window, cx| on_stop(this, window, cx)
                                        }))
                                } else {
                                    // Send / Submit Button
                                    Button::new("send-btn")
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
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Press Enter to send. Use the Mode and Approval toggles above to control tool authority.")
            )
    }
}
