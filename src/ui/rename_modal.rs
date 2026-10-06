use gpui_kit::assets::IconName;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::*;

use crate::ui::{h_flex, v_flex};

pub struct RenameModalRenderProps<'a> {
    pub is_open: bool,
    pub input_state: &'a Entity<InputState>,
}

pub struct RenameModalView;

impl RenameModalView {
    pub fn render<V: 'static>(
        props: RenameModalRenderProps<'_>,
        cx: &mut Context<V>,
        on_save: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if !props.is_open {
            return div().into_any_element();
        }

        div()
            .absolute()
            .inset_0()
            .bg(gpui::black().opacity(0.65))
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .w(gpui::px(440.0))
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_5()
                    .gap_4()
                    .child(
                        // Header
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(IconName::Pencil).small().text_color(cx.theme().primary))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().foreground)
                                            .child("Rename Thread")
                                    )
                            )
                            .child(
                                Button::new("close-rename-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::X)
                                    .on_click(cx.listener({
                                        let on_close = on_close.clone();
                                        move |this, _, window, cx| on_close(this, window, cx)
                                    }))
                            )
                    )
                    // Input Field
                    .child(
                        v_flex()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Thread Title")
                            )
                            .child(
                                Input::new(props.input_state)
                            )
                    )
                    // Action Buttons
                    .child(
                        h_flex()
                            .justify_end()
                            .gap_2()
                            .pt_2()
                            .child(
                                Button::new("cancel-rename-btn")
                                    .ghost()
                                    .small()
                                    .label("Cancel")
                                    .on_click(cx.listener({
                                        let on_close = on_close.clone();
                                        move |this, _, window, cx| on_close(this, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("save-rename-btn")
                                    .primary()
                                    .small()
                                    .icon(IconName::Check)
                                    .label("Save Title")
                                    .on_click(cx.listener({
                                        let on_save = on_save.clone();
                                        move |this, _, window, cx| on_save(this, window, cx)
                                    }))
                            )
                    )
            )
            .into_any_element()
    }
}
