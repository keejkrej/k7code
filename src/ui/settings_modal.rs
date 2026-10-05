use std::collections::HashMap;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::*;

use crate::model::{AppSettings, ProviderKind};
use crate::provider::ProviderStatus;
use crate::ui::{h_flex, v_flex};

pub struct SettingsRenderProps<'a> {
    pub is_open: bool,
    pub settings: &'a AppSettings,
    pub provider_statuses: &'a HashMap<ProviderKind, ProviderStatus>,
}

pub struct SettingsModalView;

impl SettingsModalView {
    pub fn render<V: 'static>(
        props: SettingsRenderProps<'_>,
        cx: &mut Context<V>,
        on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        if !props.is_open {
            return div().into_any_element();
        }

        div()
            .absolute()
            .inset_0()
            .bg(gpui::black().opacity(0.6))
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .w_128()
                    .max_h_128()
                    .overflow_hidden()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_6()
                    .gap_4()
                    .child(
                        // Modal Header
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(IconName::Settings).small().text_color(cx.theme().primary))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().foreground)
                                            .child("Settings & Providers")
                                    )
                            )
                            .child(
                                Button::new("close-settings-modal-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::X)
                                    .on_click(cx.listener({
                                        let on_close = on_close.clone();
                                        move |this, _, window, cx| on_close(this, window, cx)
                                    }))
                            )
                    )
                    // Detected Providers Status
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("LOCAL CLI PROVIDERS")
                            )
                            .children([
                                ProviderKind::Claude,
                                ProviderKind::Codex,
                                ProviderKind::Cursor,
                                ProviderKind::Grok,
                                ProviderKind::OpenCode,
                                ProviderKind::Antigravity,
                                ProviderKind::Ollama,
                            ].into_iter().map(|kind| {
                                let status = props.provider_statuses.get(&kind);
                                let is_available = status.map(|s| s.is_available).unwrap_or(false);
                                let path_str = status.and_then(|s| s.path.as_deref()).unwrap_or("Not found in PATH");

                                h_flex()
                                    .items_center()
                                    .justify_between()
                                    .p_2()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().secondary.opacity(0.3))
                                    .child(
                                        v_flex()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(cx.theme().foreground)
                                                    .child(kind.display_name())
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(path_str.to_string())
                                            )
                                    )
                                    .child(
                                        if is_available {
                                            Badge::new().child("Ready")
                                        } else {
                                            Badge::new().child("Missing")
                                        }
                                    )
                            }))
                    )
                    // Defaults summary
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("DEFAULTS")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child(format!("• Default Approval Policy: {}", props.settings.default_approval_policy.label()))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child(format!("• Default Runtime Mode: {}", props.settings.default_runtime_mode.label()))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child(format!("• Ollama API Endpoint: {}", props.settings.ollama_url))
                            )
                    )
            )
            .into_any_element()
    }
}
