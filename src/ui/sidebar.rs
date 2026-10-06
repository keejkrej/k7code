use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{Project, ProviderKind, Thread, TurnStatus};
use crate::ui::{h_flex, v_flex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarFilter {
    Active,
    Pinned,
    Settled,
}

pub struct SidebarRenderProps<'a> {
    pub projects: &'a [Project],
    pub active_project: Option<&'a Project>,
    pub threads: &'a [Thread],
    pub active_thread_id: Option<&'a str>,
    pub active_provider: ProviderKind,
    pub available_providers_count: usize,
    pub current_filter: SidebarFilter,
    pub search_input_state: &'a Entity<InputState>,
    pub search_query: &'a str,
}

pub struct SidebarView;

impl SidebarView {
    pub fn render<V: 'static>(
        props: SidebarRenderProps<'_>,
        cx: &mut Context<V>,
        on_select_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_new_thread: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_change_filter: impl Fn(&mut V, SidebarFilter, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_pin: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_archive: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_rename_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_delete_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_open_settings: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let pinned_count = props.threads.iter().filter(|t| t.pinned).count();
        let settled_count = props.threads.iter().filter(|t| t.archived).count();
        let active_count = props.threads.iter().filter(|t| !t.archived).count();

        let filtered_threads: Vec<&Thread> = props.threads.iter().filter(|t| {
            let matches_tab = match props.current_filter {
                SidebarFilter::Active => !t.archived,
                SidebarFilter::Pinned => t.pinned,
                SidebarFilter::Settled => t.archived,
            };

            let matches_query = if props.search_query.trim().is_empty() {
                true
            } else {
                let q = props.search_query.to_lowercase();
                t.title.to_lowercase().contains(&q)
            };

            matches_tab && matches_query
        }).collect();

        let project_name = props
            .active_project
            .map(|p| p.name.as_str())
            .unwrap_or("k7code");

        v_flex()
            .w(gpui::px(290.0))
            .h_full()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .justify_between()
            .child(
                v_flex()
                    .w_full()
                    .child(
                        // Project Scope Header matching T3 Code
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
                                    .child(
                                        div()
                                            .size_6()
                                            .rounded(cx.theme().radius)
                                            .bg(cx.theme().primary)
                                            .text_color(gpui::white())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(project_name.chars().next().unwrap_or('k').to_string().to_uppercase())
                                    )
                                    .child(
                                        v_flex()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(cx.theme().foreground)
                                                    .child(project_name.to_string())
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("Local Machine · Ready")
                                            )
                                    )
                            )
                            .child(
                                Button::new("sidebar-new-thread-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Plus)
                                    .on_click(cx.listener({
                                        let on_new = on_new_thread.clone();
                                        move |this, _, window, cx| on_new(this, window, cx)
                                    }))
                            )
                    )
                    // Fast Search Bar
                    .child(
                        div()
                            .p_2()
                            .child(
                                Input::new(props.search_input_state)
                                    .small()
                            )
                    )
                    // Section Filter Tabs with counters
                    .child(
                        h_flex()
                            .px_2()
                            .pb_2()
                            .gap_1()
                            .child(
                                Button::new("filter-active-tab")
                                    .small()
                                    .label(format!("Active ({})", active_count))
                                    .when(props.current_filter == SidebarFilter::Active, |this| this.primary())
                                    .when(props.current_filter != SidebarFilter::Active, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_filter = on_change_filter.clone();
                                        move |this, _, window, cx| on_filter(this, SidebarFilter::Active, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("filter-pinned-tab")
                                    .small()
                                    .label(format!("Pinned ({})", pinned_count))
                                    .when(props.current_filter == SidebarFilter::Pinned, |this| this.primary())
                                    .when(props.current_filter != SidebarFilter::Pinned, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_filter = on_change_filter.clone();
                                        move |this, _, window, cx| on_filter(this, SidebarFilter::Pinned, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("filter-settled-tab")
                                    .small()
                                    .label(format!("Settled ({})", settled_count))
                                    .when(props.current_filter == SidebarFilter::Settled, |this| this.primary())
                                    .when(props.current_filter != SidebarFilter::Settled, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_filter = on_change_filter.clone();
                                        move |this, _, window, cx| on_filter(this, SidebarFilter::Settled, window, cx)
                                    }))
                            )
                    )
                    // Thread Items List
                    .child(
                        v_flex()
                            .p_2()
                            .gap_1()
                            .children(filtered_threads.iter().map(|thread| {
                                let id = thread.id.clone();
                                let is_active = props.active_thread_id == Some(&thread.id);
                                let is_pinned = thread.pinned;
                                let is_settled = thread.archived;
                                let on_select = on_select_thread.clone();
                                let on_pin = on_toggle_pin.clone();
                                let on_archive = on_toggle_archive.clone();
                                let on_rename = on_rename_thread.clone();
                                let on_delete = on_delete_thread.clone();

                                let last_status = thread.turns.last().map(|t| t.status).unwrap_or(TurnStatus::Idle);
                                let (status_pill, status_color) = match last_status {
                                    TurnStatus::Idle => (None, cx.theme().muted_foreground),
                                    TurnStatus::Running => (Some("Working"), gpui::rgb(0x38bdf8).into()),
                                    TurnStatus::WaitingForApproval => (Some("Approval"), gpui::rgb(0xfbbf24).into()),
                                    TurnStatus::Completed => (Some("Done"), gpui::rgb(0x34d399).into()),
                                    TurnStatus::Interrupted | TurnStatus::Failed => (Some("Failed"), gpui::rgb(0xf87171).into()),
                                };

                                h_flex()
                                    .w_full()
                                    .p_2()
                                    .rounded(cx.theme().radius)
                                    .bg(if is_active {
                                        cx.theme().secondary
                                    } else {
                                        gpui::transparent_black().into()
                                    })
                                    .hover(|s| s.bg(cx.theme().secondary.opacity(0.6)))
                                    .cursor_pointer()
                                    .justify_between()
                                    .items_center()
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, window, cx| {
                                            on_select(this, id.clone(), window, cx);
                                        }),
                                    )
                                    .child(
                                        v_flex()
                                            .gap_0p5()
                                            .w(gpui::px(180.0))
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(if is_active {
                                                                gpui::FontWeight::SEMIBOLD
                                                            } else {
                                                                gpui::FontWeight::NORMAL
                                                            })
                                                            .text_color(if is_active {
                                                                cx.theme().foreground
                                                            } else {
                                                                cx.theme().muted_foreground
                                                            })
                                                            .child(thread.title.clone())
                                                    )
                                                    .when(is_pinned, |this| {
                                                        this.child(Icon::new(IconName::Pin).small().text_color(cx.theme().primary))
                                                    })
                                                    .when(is_settled, |this| {
                                                        this.child(Icon::new(IconName::Archive).small().text_color(cx.theme().muted_foreground))
                                                    })
                                            )
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground.opacity(0.8))
                                                            .child(thread.created_at.format("%b %d, %H:%M").to_string())
                                                    )
                                                    .when_some(status_pill, |this, label| {
                                                        this.child(
                                                            div()
                                                                .px_1p5()
                                                                .py_0p5()
                                                                .rounded_full()
                                                                .bg(status_color.opacity(0.18))
                                                                .text_xs()
                                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                                .text_color(status_color)
                                                                .child(label)
                                                        )
                                                    })
                                            )
                                    )
                                    .child(
                                        // Quick Action Icon Buttons on Row
                                        h_flex()
                                            .items_center()
                                            .gap_0p5()
                                            .child(
                                                Button::new(format!("pin-btn-{}", thread.id))
                                                    .ghost()
                                                    .small()
                                                    .icon(if is_pinned { IconName::PinOff } else { IconName::Pin })
                                                    .on_click(cx.listener({
                                                        let tid = thread.id.clone();
                                                        let on_pin = on_pin.clone();
                                                        move |this, _, window, cx| on_pin(this, tid.clone(), window, cx)
                                                    }))
                                            )
                                            .child(
                                                Button::new(format!("rename-btn-{}", thread.id))
                                                    .ghost()
                                                    .small()
                                                    .icon(IconName::Pencil)
                                                    .on_click(cx.listener({
                                                        let tid = thread.id.clone();
                                                        let on_rename = on_rename.clone();
                                                        move |this, _, window, cx| on_rename(this, tid.clone(), window, cx)
                                                    }))
                                            )
                                            .child(
                                                Button::new(format!("archive-btn-{}", thread.id))
                                                    .ghost()
                                                    .small()
                                                    .icon(if is_settled { IconName::RotateCcw } else { IconName::Archive })
                                                    .on_click(cx.listener({
                                                        let tid = thread.id.clone();
                                                        let on_archive = on_archive.clone();
                                                        move |this, _, window, cx| on_archive(this, tid.clone(), window, cx)
                                                    }))
                                            )
                                            .child(
                                                Button::new(format!("delete-btn-{}", thread.id))
                                                    .ghost()
                                                    .small()
                                                    .icon(IconName::Trash)
                                                    .on_click(cx.listener({
                                                        let tid = thread.id.clone();
                                                        let on_delete = on_delete.clone();
                                                        move |this, _, window, cx| on_delete(this, tid.clone(), window, cx)
                                                    }))
                                            )
                                    )
                            }))
                    )
            )
            // Bottom Chrome: Provider status & High Refresh rate badge
            .child(
                v_flex()
                    .p_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().secondary.opacity(0.15))
                    .gap_2()
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
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(cx.theme().foreground)
                                            .child(props.active_provider.display_name())
                                    )
                            )
                            .child(
                                Badge::new()
                                    .small()
                                    .child(format!("{} Ready", props.available_providers_count))
                            )
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Engine: GPUI Kit · 120 FPS")
                            )
                            .child(
                                Button::new("bottom-settings-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Settings)
                                    .on_click(cx.listener({
                                        let on_open_settings = on_open_settings.clone();
                                        move |this, _, window, cx| on_open_settings(this, window, cx)
                                    }))
                            )
                    )
            )
    }
}
