use std::collections::HashMap;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{AppData, ApprovalPolicy, ProviderKind, RuntimeMode};
use crate::provider::ProviderStatus;
use crate::ui::{h_flex, v_flex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Providers,
    Policies,
    Projects,
    About,
}

pub struct SettingsRenderProps<'a> {
    pub is_open: bool,
    pub current_tab: SettingsTab,
    pub app_data: &'a AppData,
    pub provider_statuses: &'a HashMap<ProviderKind, ProviderStatus>,
}

pub struct SettingsModalView;

impl SettingsModalView {
    pub fn render<V: 'static>(
        props: SettingsRenderProps<'_>,
        cx: &mut Context<V>,
        on_tab_change: impl Fn(&mut V, SettingsTab, &mut Window, &mut Context<V>) + 'static + Clone,
        on_rescan_providers: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_change_default_mode: impl Fn(&mut V, RuntimeMode, &mut Window, &mut Context<V>) + 'static + Clone,
        on_change_default_policy: impl Fn(&mut V, ApprovalPolicy, &mut Window, &mut Context<V>) + 'static + Clone,
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
                    .w(gpui::px(640.0))
                    .max_h(gpui::px(600.0))
                    .overflow_hidden()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_5()
                    .gap_4()
                    .child(
                        // Modal Header
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
                                    .child(Icon::new(IconName::Settings).small().text_color(cx.theme().primary))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().foreground)
                                            .child("Settings & Configuration")
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
                    // Tabs Header
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("tab-providers")
                                    .small()
                                    .label("CLI Providers")
                                    .when(props.current_tab == SettingsTab::Providers, |this| this.primary())
                                    .when(props.current_tab != SettingsTab::Providers, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_tab_change = on_tab_change.clone();
                                        move |this, _, window, cx| on_tab_change(this, SettingsTab::Providers, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("tab-policies")
                                    .small()
                                    .label("Execution Policies")
                                    .when(props.current_tab == SettingsTab::Policies, |this| this.primary())
                                    .when(props.current_tab != SettingsTab::Policies, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_tab_change = on_tab_change.clone();
                                        move |this, _, window, cx| on_tab_change(this, SettingsTab::Policies, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("tab-projects")
                                    .small()
                                    .label("Projects & Storage")
                                    .when(props.current_tab == SettingsTab::Projects, |this| this.primary())
                                    .when(props.current_tab != SettingsTab::Projects, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_tab_change = on_tab_change.clone();
                                        move |this, _, window, cx| on_tab_change(this, SettingsTab::Projects, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("tab-about")
                                    .small()
                                    .label("About")
                                    .when(props.current_tab == SettingsTab::About, |this| this.primary())
                                    .when(props.current_tab != SettingsTab::About, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_tab_change = on_tab_change.clone();
                                        move |this, _, window, cx| on_tab_change(this, SettingsTab::About, window, cx)
                                    }))
                            )
                    )
                    // Tab Content Body
                    .child(
                        match props.current_tab {
                            SettingsTab::Providers => Self::render_providers_tab(props, cx, on_rescan_providers).into_any_element(),
                            SettingsTab::Policies => Self::render_policies_tab(props, cx, on_change_default_mode, on_change_default_policy).into_any_element(),
                            SettingsTab::Projects => Self::render_projects_tab(props, cx).into_any_element(),
                            SettingsTab::About => Self::render_about_tab(cx).into_any_element(),
                        }
                    )
            )
            .into_any_element()
    }

    fn render_providers_tab<V: 'static>(
        props: SettingsRenderProps<'_>,
        cx: &mut Context<V>,
        on_rescan_providers: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let providers = [
            ProviderKind::Claude,
            ProviderKind::Codex,
            ProviderKind::Cursor,
            ProviderKind::Grok,
            ProviderKind::OpenCode,
            ProviderKind::Antigravity,
            ProviderKind::Ollama,
        ];

        v_flex()
            .gap_3()
            .max_h(gpui::px(420.0))
            .overflow_hidden()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Detected agents and CLI tools available on your system PATH")
                    )
                    .child(
                        Button::new("rescan-btn")
                            .secondary()
                            .small()
                            .icon(IconName::RotateCw)
                            .label("Re-scan PATH")
                            .on_click(cx.listener({
                                let on_rescan = on_rescan_providers.clone();
                                move |this, _, window, cx| on_rescan(this, window, cx)
                            }))
                    )
            )
            .children(providers.into_iter().map(|kind| {
                let status = props.provider_statuses.get(&kind);
                let is_available = status.map(|s| s.is_available).unwrap_or(false);
                let path_str = status.and_then(|s| s.path.as_deref()).unwrap_or("Not found in PATH");
                let ver_str = status.and_then(|s| s.version.as_deref()).unwrap_or("");

                h_flex()
                    .items_center()
                    .justify_between()
                    .p_2p5()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary.opacity(0.3))
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(cx.theme().foreground)
                                            .child(kind.display_name())
                                    )
                                    .when(!ver_str.is_empty(), |this| {
                                        this.child(
                                            Badge::new()
                                                .small()
                                                .child(ver_str.to_string())
                                        )
                                    })
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
                            Badge::new().small().child("Ready")
                        } else {
                            Badge::new().small().child("Missing")
                        }
                    )
            }))
    }

    fn render_policies_tab<V: 'static>(
        props: SettingsRenderProps<'_>,
        cx: &mut Context<V>,
        on_change_default_mode: impl Fn(&mut V, RuntimeMode, &mut Window, &mut Context<V>) + 'static + Clone,
        on_change_default_policy: impl Fn(&mut V, ApprovalPolicy, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let modes = [
            RuntimeMode::Supervised,
            RuntimeMode::AutoAcceptEdits,
            RuntimeMode::Auto,
            RuntimeMode::FullAccess,
        ];

        let policies = [
            ApprovalPolicy::Untrusted,
            ApprovalPolicy::OnRequest,
            ApprovalPolicy::OnFailure,
            ApprovalPolicy::Never,
        ];

        v_flex()
            .gap_4()
            .max_h(gpui::px(420.0))
            .overflow_hidden()
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("DEFAULT RUNTIME MODE")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Controls how autonomously providers are permitted to propose file edits and commands")
                    )
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .children(modes.into_iter().map(|m| {
                                let is_active = props.app_data.settings.default_runtime_mode == m;
                                let on_change = on_change_default_mode.clone();

                                Button::new(format!("set-mode-{:?}", m))
                                    .small()
                                    .label(m.label())
                                    .when(is_active, |this| this.primary())
                                    .when(!is_active, |this| this.secondary())
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_change(this, m, window, cx);
                                    }))
                            }))
                    )
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("DEFAULT APPROVAL POLICY")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("When user confirmation is required before system commands execute")
                    )
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .children(policies.into_iter().map(|p| {
                                let is_active = props.app_data.settings.default_approval_policy == p;
                                let on_change = on_change_default_policy.clone();

                                Button::new(format!("set-policy-{:?}", p))
                                    .small()
                                    .label(p.label())
                                    .when(is_active, |this| this.primary())
                                    .when(!is_active, |this| this.secondary())
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        on_change(this, p, window, cx);
                                    }))
                            }))
                    )
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("OLLAMA LOCAL API")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("Endpoint: {}", props.app_data.settings.ollama_url))
                    )
            )
    }

    fn render_projects_tab<V: 'static>(
        props: SettingsRenderProps<'_>,
        cx: &mut Context<V>,
    ) -> impl IntoElement {
        let storage_path = dirs::home_dir()
            .map(|p| p.join(".k7code").join("data.json"))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "~/.k7code/data.json".to_string());

        v_flex()
            .gap_3()
            .max_h(gpui::px(420.0))
            .overflow_hidden()
            .child(
                v_flex()
                    .p_3()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary.opacity(0.3))
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("PERSISTENT STORAGE")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("Path: {storage_path}"))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("Total Projects: {} · Total Threads: {}", props.app_data.projects.len(), props.app_data.threads.len()))
                    )
            )
            .child(
                v_flex()
                    .gap_1p5()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("REGISTERED PROJECTS")
                    )
                    .children(props.app_data.projects.iter().map(|proj| {
                        let branch_str = proj.git_branch.clone().unwrap_or_else(|| "no git".to_string());

                        h_flex()
                            .items_center()
                            .justify_between()
                            .p_2()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().secondary.opacity(0.2))
                            .child(
                                v_flex()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(cx.theme().foreground)
                                            .child(proj.name.clone())
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(proj.path.to_string_lossy().to_string())
                                    )
                            )
                            .child(
                                Badge::new()
                                    .small()
                                    .child(branch_str)
                            )
                    }))
            )
    }

    fn render_about_tab<V: 'static>(cx: &mut Context<V>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .max_h(gpui::px(420.0))
            .overflow_hidden()
            .child(
                v_flex()
                    .p_4()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary.opacity(0.3))
                    .gap_2()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(IconName::Sparkles).text_color(cx.theme().primary))
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("k7code Desktop Agent")
                            )
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .line_height(gpui::relative(1.4))
                            .child("k7code is a high-performance, 100% native Rust desktop AI workspace implementing the T3 Code experience. Built directly on GPUI Kit for instant 120 FPS rendering, zero-overhead process execution, and seamless local CLI provider orchestration.")
                    )
            )
            .child(
                v_flex()
                    .p_3()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().secondary.opacity(0.2))
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .child("• Native Engine: GPUI Kit 0.7.1 + Rust 2024")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .child("• Rendering: Hardware Accelerated Direct3D / Metal / Vulkan")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .child("• Process Management: Native OS thread loops with atomic cancellation")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .child("• Compatibility: Reference architecture parity with T3 Code")
                    )
            )
    }
}
