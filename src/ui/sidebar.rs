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
        on_filter_change: impl Fn(&mut V, SidebarFilter, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_pin: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_archive: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_rename_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_delete_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_open_settings: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let active_project_id = props.active_project.map(|p| p.id.as_str());

        // Count tabs
        let active_count = props
            .threads
            .iter()
            .filter(|t| Some(t.project_id.as_str()) == active_project_id && !t.archived)
            .count();
        let pinned_count = props
            .threads
            .iter()
            .filter(|t| Some(t.project_id.as_str()) == active_project_id && t.pinned && !t.archived)
            .count();
        let settled_count = props
            .threads
            .iter()
            .filter(|t| Some(t.project_id.as_str()) == active_project_id && t.archived)
            .count();

        // Filter threads
        let query = props.search_query.trim().to_lowercase();
        let filtered_threads: Vec<_> = props
            .threads
            .iter()
            .filter(|t| {
                if Some(t.project_id.as_str()) != active_project_id {
                    return false;
                }
                match props.current_filter {
                    SidebarFilter::Active => !t.archived,
                    SidebarFilter::Pinned => t.pinned && !t.archived,
                    SidebarFilter::Settled => t.archived,
                }
            })
            .filter(|t| {
                if query.is_empty() {
                    return true;
                }
                t.title.to_lowercase().contains(&query)
                    || t.turns.iter().any(|turn| turn.user_prompt.to_lowercase().contains(&query))
            })
            .collect();

        v_flex()
            .w(gpui::px(280.0))
            .h_full()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .justify_between()
            .child(
                v_flex()
                    .w_full()
                    .child(
                        // Top Brand Bar with New Thread Action
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
                                        Icon::new(IconName::Sparkles)
                                            .small()
                                            .text_color(cx.theme().primary)
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(cx.theme().foreground)
                                            .child("T3 Code")
                                    )
                                    .child(
                                        Badge::new()
                                            .small()
                                            .child("Local")
                                    )
                            )
                            .child(
                                Button::new("new-thread-btn")
                                    .primary()
                                    .small()
                                    .icon(IconName::Plus)
                                    .on_click(cx.listener({
                                        let on_new_thread = on_new_thread.clone();
                                        move |this, _, window, cx| on_new_thread(this, window, cx)
                                    }))
                            )
                    )
                    // Active Workspace Card
                    .when_some(props.active_project, |this, project| {
                        let branch_str = project.git_branch.clone().unwrap_or_else(|| "no-git".to_string());
                        let path_str = project.path.to_string_lossy().to_string();

                        this.child(
                            v_flex()
                                .p_2p5()
                                .mx_2()
                                .my_2()
                                .rounded(cx.theme().radius)
                                .bg(cx.theme().secondary.opacity(0.35))
                                .border_1()
                                .border_color(cx.theme().border.opacity(0.6))
                                .gap_1()
                                .child(
                                    h_flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            h_flex()
                                                .items_center()
                                                .gap_1p5()
                                                .child(Icon::new(IconName::Folder).small().text_color(cx.theme().primary))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                                        .text_color(cx.theme().foreground)
                                                        .child(project.name.clone())
                                                )
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(branch_str)
                                        )
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(path_str)
                                )
                        )
                    })
                    // Quick Search Bar
                    .child(
                        div()
                            .px_2()
                            .pb_2()
                            .child(
                                Input::new(props.search_input_state)
                            )
                    )
                    // Filter Tabs: Active / Pinned / Settled
                    .child(
                        h_flex()
                            .px_2()
                            .pb_2()
                            .gap_1()
                            .child(
                                Button::new("filter-active")
                                    .small()
                                    .label(format!("Active ({active_count})"))
                                    .when(props.current_filter == SidebarFilter::Active, |this| this.primary())
                                    .when(props.current_filter != SidebarFilter::Active, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_filter_change = on_filter_change.clone();
                                        move |this, _, window, cx| on_filter_change(this, SidebarFilter::Active, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("filter-pinned")
                                    .small()
                                    .label(format!("Pinned ({pinned_count})"))
                                    .when(props.current_filter == SidebarFilter::Pinned, |this| this.primary())
                                    .when(props.current_filter != SidebarFilter::Pinned, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_filter_change = on_filter_change.clone();
                                        move |this, _, window, cx| on_filter_change(this, SidebarFilter::Pinned, window, cx)
                                    }))
                            )
                            .child(
                                Button::new("filter-settled")
                                    .small()
                                    .label(format!("Settled ({settled_count})"))
                                    .when(props.current_filter == SidebarFilter::Settled, |this| this.primary())
                                    .when(props.current_filter != SidebarFilter::Settled, |this| this.ghost())
                                    .on_click(cx.listener({
                                        let on_filter_change = on_filter_change.clone();
                                        move |this, _, window, cx| on_filter_change(this, SidebarFilter::Settled, window, cx)
                                    }))
                            )
                    )
                    // Thread Cards List
                    .child(
                        v_flex()
                            .px_2()
                            .gap_1()
                            .max_h(gpui::px(520.0))
                            .overflow_hidden()
                            .when(filtered_threads.is_empty(), |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("No threads match this filter.")
                                )
                            })
                            .children(filtered_threads.into_iter().map(|thread| {
                                let is_active = props.active_thread_id == Some(&thread.id);
                                let thread_id = thread.id.clone();
                                let thread_id_for_select = thread.id.clone();
                                let thread_id_for_pin = thread.id.clone();
                                let thread_id_for_archive = thread.id.clone();
                                let thread_id_for_rename = thread.id.clone();
                                let thread_id_for_delete = thread.id.clone();

                                let last_turn = thread.turns.last();
                                let is_running = last_turn.map(|t| t.status == TurnStatus::Running).unwrap_or(false);
                                let is_waiting_approval = last_turn.map(|t| t.status == TurnStatus::WaitingForApproval).unwrap_or(false);

                                let on_select_thread = on_select_thread.clone();
                                let on_toggle_pin = on_toggle_pin.clone();
                                let on_toggle_archive = on_toggle_archive.clone();
                                let on_rename_thread = on_rename_thread.clone();
                                let on_delete_thread = on_delete_thread.clone();

                                v_flex()
                                    .w_full()
                                    .p_2()
                                    .rounded(cx.theme().radius)
                                    .border_1()
                                    .border_color(if is_active {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().border.opacity(0.4)
                                    })
                                    .bg(if is_active {
                                        cx.theme().primary.opacity(0.12)
                                    } else {
                                        cx.theme().secondary.opacity(0.2)
                                    })
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, window, cx| {
                                            on_select_thread(this, thread_id_for_select.clone(), window, cx);
                                        }),
                                    )
                                    .child(
                                        // Title and status badges
                                        h_flex()
                                            .w_full()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1p5()
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
                                                            .child(thread.title.clone())
                                                    )
                                            )
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .when(is_running, |this| {
                                                        this.child(
                                                            Badge::new()
                                                                .small()
                                                                .child("Running")
                                                        )
                                                    })
                                                    .when(is_waiting_approval, |this| {
                                                        this.child(
                                                            Badge::new()
                                                                .small()
                                                                .child("Action Req")
                                                        )
                                                    })
                                                    .when(thread.pinned, |this| {
                                                        this.child(
                                                            Icon::new(IconName::Pin)
                                                                .small()
                                                                .text_color(cx.theme().primary)
                                                        )
                                                    })
                                            )
                                    )
                                    .child(
                                        // Metadata subtitle & Quick card actions
                                        h_flex()
                                            .w_full()
                                            .items_center()
                                            .justify_between()
                                            .pt_1()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!("{} · {} turns", thread.model, thread.turns.len()))
                                            )
                                            .child(
                                                h_flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(
                                                        Button::new(format!("pin-btn-{}", thread_id))
                                                            .ghost()
                                                            .small()
                                                            .icon(IconName::Pin)
                                                            .on_click(cx.listener({
                                                                let on_toggle_pin = on_toggle_pin.clone();
                                                                move |this, _, window, cx| on_toggle_pin(this, thread_id_for_pin.clone(), window, cx)
                                                            }))
                                                    )
                                                    .child(
                                                        Button::new(format!("archive-btn-{}", thread_id))
                                                            .ghost()
                                                            .small()
                                                            .icon(IconName::Archive)
                                                            .on_click(cx.listener({
                                                                let on_toggle_archive = on_toggle_archive.clone();
                                                                move |this, _, window, cx| on_toggle_archive(this, thread_id_for_archive.clone(), window, cx)
                                                            }))
                                                    )
                                                    .child(
                                                        Button::new(format!("rename-btn-{}", thread_id))
                                                            .ghost()
                                                            .small()
                                                            .icon(IconName::Pencil)
                                                            .on_click(cx.listener({
                                                                let on_rename_thread = on_rename_thread.clone();
                                                                move |this, _, window, cx| on_rename_thread(this, thread_id_for_rename.clone(), window, cx)
                                                            }))
                                                    )
                                                    .child(
                                                        Button::new(format!("delete-btn-{}", thread_id))
                                                            .ghost()
                                                            .small()
                                                            .icon(IconName::Trash)
                                                            .on_click(cx.listener({
                                                                let on_delete_thread = on_delete_thread.clone();
                                                                move |this, _, window, cx| on_delete_thread(this, thread_id_for_delete.clone(), window, cx)
                                                            }))
                                                    )
                                            )
                                    )
                            }))
                    )
            )
            // Sidebar Footer Chrome
            .child(
                v_flex()
                    .w_full()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .p_3()
                    .gap_2()
                    .child(
                        // Active CLI Provider indicator
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
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
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} ready", props.available_providers_count))
                            )
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                Badge::new()
                                    .small()
                                    .child("Rust GPUI · 120 FPS")
                            )
                            .child(
                                Button::new("sidebar-settings-btn")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Settings)
                                    .label("Settings")
                                    .on_click(cx.listener({
                                        let on_open_settings = on_open_settings.clone();
                                        move |this, _, window, cx| on_open_settings(this, window, cx)
                                    }))
                            )
                    )
            )
    }
}
