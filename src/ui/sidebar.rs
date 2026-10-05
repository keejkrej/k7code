use gpui_kit::assets::IconName;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    Icon, Sizable, ActiveTheme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::model::{Project, Thread};
use crate::ui::{h_flex, v_flex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadFilterTab {
    Active,
    Pinned,
    Archived,
}

pub struct SidebarRenderProps<'a> {
    pub projects: &'a [Project],
    pub active_project: Option<&'a Project>,
    pub threads: &'a [Thread],
    pub active_thread_id: Option<&'a str>,
    pub selected_tab: ThreadFilterTab,
    pub detected_providers_count: usize,
}

pub struct SidebarView;

impl SidebarView {
    pub fn render<V: 'static>(
        props: SidebarRenderProps<'_>,
        cx: &mut Context<V>,
        on_select_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_new_thread: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_pin: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_archive: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_delete_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_change_tab: impl Fn(&mut V, ThreadFilterTab, &mut Window, &mut Context<V>) + 'static + Clone,
        on_open_settings: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let active_pid = props.active_project.map(|p| p.id.as_str());

        // Filter threads according to current project and filter tab
        let filtered_threads: Vec<&Thread> = props
            .threads
            .iter()
            .filter(|t| Some(t.project_id.as_str()) == active_pid)
            .filter(|t| match props.selected_tab {
                ThreadFilterTab::Active => !t.archived,
                ThreadFilterTab::Pinned => t.pinned && !t.archived,
                ThreadFilterTab::Archived => t.archived,
            })
            .collect();

        v_flex()
            .w_72()
            .h_full()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .justify_between()
            .child(
                v_flex()
                    .w_full()
                    .child(
                        // Project Selector Header
                        v_flex()
                            .p_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .gap_1()
                            .child(
                                h_flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                Icon::new(IconName::Folder)
                                                    .small()
                                                    .text_color(cx.theme().primary)
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(cx.theme().foreground)
                                                    .child(
                                                        props
                                                            .active_project
                                                            .map(|p| p.name.clone())
                                                            .unwrap_or_else(|| "No Project".to_string()),
                                                    )
                                            )
                                    )
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        props
                                            .active_project
                                            .map(|p| p.path.display().to_string())
                                            .unwrap_or_default(),
                                    )
                            )
                    )
                    // New Thread Action Button
                    .child(
                        v_flex()
                            .p_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .gap_2()
                            .child(
                                Button::new("new-thread-btn")
                                    .primary()
                                    .w_full()
                                    .icon(IconName::Plus)
                                    .label("New Thread")
                                    .on_click(cx.listener({
                                        let on_new_thread = on_new_thread.clone();
                                        move |this, _, window, cx| on_new_thread(this, window, cx)
                                    }))
                            )
                            // Filter Tabs: Active | Pinned | Archived
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_1()
                                    .child(
                                        Button::new("tab-active")
                                            .when(props.selected_tab == ThreadFilterTab::Active, |b| b.secondary())
                                            .when(props.selected_tab != ThreadFilterTab::Active, |b| b.ghost())
                                            .small()
                                            .flex_1()
                                            .label("Active")
                                            .on_click(cx.listener({
                                                let on_change_tab = on_change_tab.clone();
                                                move |this, _, window, cx| on_change_tab(this, ThreadFilterTab::Active, window, cx)
                                            }))
                                    )
                                    .child(
                                        Button::new("tab-pinned")
                                            .when(props.selected_tab == ThreadFilterTab::Pinned, |b| b.secondary())
                                            .when(props.selected_tab != ThreadFilterTab::Pinned, |b| b.ghost())
                                            .small()
                                            .flex_1()
                                            .label("Pinned")
                                            .on_click(cx.listener({
                                                let on_change_tab = on_change_tab.clone();
                                                move |this, _, window, cx| on_change_tab(this, ThreadFilterTab::Pinned, window, cx)
                                            }))
                                    )
                                    .child(
                                        Button::new("tab-archived")
                                            .when(props.selected_tab == ThreadFilterTab::Archived, |b| b.secondary())
                                            .when(props.selected_tab != ThreadFilterTab::Archived, |b| b.ghost())
                                            .small()
                                            .flex_1()
                                            .label("Archive")
                                            .on_click(cx.listener({
                                                let on_change_tab = on_change_tab.clone();
                                                move |this, _, window, cx| on_change_tab(this, ThreadFilterTab::Archived, window, cx)
                                            }))
                                    )
                            )
                    )
                    // Thread Items List
                    .child(
                        v_flex()
                            .p_2()
                            .gap_1()
                            .when(filtered_threads.is_empty(), |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("No threads in this view.")
                                )
                            })
                            .children(filtered_threads.into_iter().map(|thread| {
                                let is_active = props.active_thread_id == Some(&thread.id);
                                let is_pinned = thread.pinned;
                                let is_archived = thread.archived;
                                let is_running = thread.is_running();
                                let tid = thread.id.clone();
                                let tid_pin = thread.id.clone();
                                let tid_arc = thread.id.clone();
                                let tid_del = thread.id.clone();

                                h_flex()
                                    .w_full()
                                    .px_2()
                                    .py_1p5()
                                    .rounded(cx.theme().radius)
                                    .items_center()
                                    .justify_between()
                                    .when(is_active, |this| {
                                        this.bg(cx.theme().accent.opacity(0.15))
                                    })
                                    .hover(|s| s.bg(cx.theme().secondary.opacity(0.3)))
                                    .child(
                                        h_flex()
                                            .flex_1()
                                            .items_center()
                                            .gap_2()
                                            .cursor_pointer()
                                            .child(
                                                if is_running {
                                                    Icon::new(IconName::RotateCw)
                                                        .small()
                                                        .text_color(cx.theme().primary)
                                                } else if is_pinned {
                                                    Icon::new(IconName::Pin)
                                                        .small()
                                                        .text_color(cx.theme().primary)
                                                } else {
                                                    Icon::new(IconName::MessageSquare)
                                                        .small()
                                                        .text_color(cx.theme().muted_foreground)
                                                }
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(if is_active {
                                                        cx.theme().primary
                                                    } else {
                                                        cx.theme().foreground
                                                    })
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .child(thread.title.clone())
                                            )
                                    )
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_0p5()
                                            // Pin action button
                                            .child(
                                                Button::new(format!("pin-{}", tid_pin))
                                                    .ghost()
                                                    .small()
                                                    .icon(IconName::Pin)
                                                    .on_click(cx.listener({
                                                        let on_toggle_pin = on_toggle_pin.clone();
                                                        move |this, _, window, cx| on_toggle_pin(this, tid_pin.clone(), window, cx)
                                                    }))
                                            )
                                            // Archive action button
                                            .child(
                                                Button::new(format!("arc-{}", tid_arc))
                                                    .ghost()
                                                    .small()
                                                    .icon(if is_archived { IconName::FolderOpen } else { IconName::Archive })
                                                    .on_click(cx.listener({
                                                        let on_toggle_archive = on_toggle_archive.clone();
                                                        move |this, _, window, cx| on_toggle_archive(this, tid_arc.clone(), window, cx)
                                                    }))
                                            )
                                            // Delete action button
                                            .child(
                                                Button::new(format!("del-{}", tid_del))
                                                    .ghost()
                                                    .small()
                                                    .icon(IconName::Trash)
                                                    .on_click(cx.listener({
                                                        let on_delete_thread = on_delete_thread.clone();
                                                        move |this, _, window, cx| on_delete_thread(this, tid_del.clone(), window, cx)
                                                    }))
                                            )
                                    )
                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener({
                                        let on_select_thread = on_select_thread.clone();
                                        move |this, _, window, cx| on_select_thread(this, tid.clone(), window, cx)
                                    }))
                            }))
                    )
            )
            // Footer: Detected Local Providers & Settings Modal Trigger
            .child(
                v_flex()
                    .p_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
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
                                        Icon::new(IconName::Cpu)
                                            .small()
                                            .text_color(cx.theme().muted_foreground)
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!("{} Providers Detected", props.detected_providers_count))
                                    )
                            )
                            .child(
                                Button::new("settings-btn")
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
            .into_any_element()
    }
}
