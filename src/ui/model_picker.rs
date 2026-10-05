use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::ProviderKind;
use crate::ui::{h_flex, v_flex};

pub struct ModelPickerRenderProps<'a> {
    pub is_open: bool,
    pub current_provider: ProviderKind,
    pub current_model: &'a str,
}

pub struct ModelPickerModalView;

impl ModelPickerModalView {
    pub fn render<V: 'static>(
        props: ModelPickerRenderProps<'_>,
        cx: &mut Context<V>,
        on_select: impl Fn(&mut V, ProviderKind, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if !props.is_open {
            return div().into_any_element();
        }

        let providers = [
            ProviderKind::Claude,
            ProviderKind::Codex,
            ProviderKind::Cursor,
            ProviderKind::Grok,
            ProviderKind::OpenCode,
            ProviderKind::Antigravity,
            ProviderKind::Ollama,
        ];

        div()
            .absolute()
            .inset_0()
            .bg(gpui::black().opacity(0.65))
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .w(gpui::px(640.0))
                    .max_h(gpui::px(600.0))
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
                            .pb_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(IconName::Sparkles).small().text_color(cx.theme().primary))
                                    .child(
                                        v_flex()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(cx.theme().foreground)
                                                    .child("Select AI Model & Provider")
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("Pick an available model to power instructions and tool execution")
                                            )
                                    )
                            )
                            .child(
                                Button::new("close-model-picker-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::X)
                                    .on_click(cx.listener({
                                        let on_close = on_close.clone();
                                        move |this, _, window, cx| on_close(this, window, cx)
                                    }))
                            )
                    )
                    // Providers and models scrollable list
                    .child(
                        v_flex()
                            .gap_4()
                            .max_h(gpui::px(460.0))
                            .overflow_hidden()
                            .children(providers.into_iter().map(|provider| {
                                let models = provider.available_models();
                                let current_provider = props.current_provider;
                                let current_model = props.current_model.to_string();

                                v_flex()
                                    .gap_2()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(cx.theme().primary)
                                                    .child(provider.display_name())
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!("({})", provider.default_binary_name()))
                                            )
                                    )
                                    .child(
                                        h_flex()
                                            .flex_wrap()
                                            .gap_2()
                                            .children(models.into_iter().map(|model| {
                                                let is_active = provider == current_provider && model.id == current_model;
                                                let model_id = model.id.clone();
                                                let model_name = model.name.clone();
                                                let supports_thinking = model.supports_thinking;
                                                let on_select = on_select.clone();

                                                h_flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .px_3()
                                                    .py_1p5()
                                                    .rounded(cx.theme().radius)
                                                    .border_1()
                                                    .border_color(if is_active {
                                                        cx.theme().primary
                                                    } else {
                                                        cx.theme().border
                                                    })
                                                    .bg(if is_active {
                                                        cx.theme().primary.opacity(0.15)
                                                    } else {
                                                        cx.theme().secondary.opacity(0.3)
                                                    })
                                                    .cursor_pointer()
                                                    .on_mouse_down(
                                                        gpui::MouseButton::Left,
                                                        cx.listener(move |this, _, window, cx| {
                                                            on_select(this, provider, model_id.clone(), window, cx);
                                                        }),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(if is_active {
                                                                gpui::FontWeight::SEMIBOLD
                                                            } else {
                                                                gpui::FontWeight::MEDIUM
                                                            })
                                                            .text_color(if is_active {
                                                                cx.theme().primary
                                                            } else {
                                                                cx.theme().foreground
                                                            })
                                                            .child(model_name)
                                                    )
                                                    .when(supports_thinking, |this| {
                                                        this.child(
                                                            Badge::new()
                                                                .small()
                                                                .child("Thinking")
                                                        )
                                                    })
                                                    .when(is_active, |this| {
                                                        this.child(
                                                            Icon::new(IconName::Check)
                                                                .small()
                                                                .text_color(cx.theme().primary)
                                                        )
                                                    })
                                            }))
                                    )
                            }))
                    )
            )
            .into_any_element()
    }
}
