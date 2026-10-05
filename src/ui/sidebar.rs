use gpui_kit::assets::IconName;
use gpui_kit::component::{
    badge::Badge,
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
    pub active_provider_name: &'a str,
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
        let project_name = props
            .active_project
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "No Project".to_string());
        let branch_name = props
            .active_project
            .and_then(|p| p.git_branch.clone())
            .unwrap_or_else(|| "main".to_string());

        // Partition threads for active project
        let mut pinned_threads = Vec::new();
        let mut active_threads = Vec::new();
        let mut settled_threads = Vec::new();

        for t in props.threads.iter().filter(|t| Some(t.project_id.as_str()) == active_pid) {
            if t.archived {
                settled_threads.push(t);
            } else if t.pinned {
                pinned_threads.push(t);
            } else {
                active_threads.push(t);
            }
        }

        // Build thread cards containers without closure captures
        let mut thread_card_elements = Vec::new();

        match props.selected_tab {
            ThreadFilterTab::Active => {
                if !pinned_threads.is_empty() {
                    thread_card_elements.push(
                        h_flex()
                            .items_center()
                            .gap_1()
                            .px_2()
                            .py_1()
                            .child(Icon::new(IconName::Pin).small().text_color(cx.theme().muted_foreground))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("PINNED")
                            )
                            .into_any_element(),
                    );
                    for thread in &pinned_threads {
                        thread_card_elements.push(
                            Self::render_thread_card(
                                thread,
                                props.active_thread_id == Some(&thread.id),
                                cx,
                                on_select_thread.clone(),
                                on_toggle_pin.clone(),
                                on_toggle_archive.clone(),
                                on_delete_thread.clone(),
                            )
                            .into_any_element(),
                        );
                    }
                    thread_card_elements.push(
                        div()
                            .px_2()
                            .py_1()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().muted_foreground)
                            .child("ACTIVE")
                            .into_any_element(),
                    );
                }

                for thread in &active_threads {
                    thread_card_elements.push(
                        Self::render_thread_card(
                            thread,
                            props.active_thread_id == Some(&thread.id),
                            cx,
                            on_select_thread.clone(),
                            on_toggle_pin.clone(),
                            on_toggle_archive.clone(),
                            on_delete_thread.clone(),
                        )
                        .into_any_element(),
                    );
                }
            }
            ThreadFilterTab::Pinned => {
                for thread in &pinned_threads {
                    thread_card_elements.push(
                        Self::render_thread_card(
                            thread,
                            props.active_thread_id == Some(&thread.id),
                            cx,
                            on_select_thread.clone(),
                            on_toggle_pin.clone(),
                            on_toggle_archive.clone(),
                            on_delete_thread.clone(),
                        )
                        .into_any_element(),
                    );
                }
            }
            ThreadFilterTab::Archived => {
                for thread in &settled_threads {
                    thread_card_elements.push(
                        Self::render_thread_card(
                            thread,
                            props.active_thread_id == Some(&thread.id),
                            cx,
                            on_select_thread.clone(),
                            on_toggle_pin.clone(),
                            on_toggle_archive.clone(),
                            on_delete_thread.clone(),
                        )
                        .into_any_element(),
                    );
                }
            }
        }

        let is_current_tab_empty = thread_card_elements.is_empty();

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
                        // 1. T3 Code Titlebar Chrome & Brand Header
                        v_flex()
                            .p_3()
                            .border_b_1()
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
                                                div()
                                                    .text_base()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(cx.theme().foreground)
                                                    .child("T3")
                                            )
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_weight(gpui::FontWeight::NORMAL)
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("Code")
                                            )
                                            .child(
                                                Badge::new()
                                                    .small()
                                                    .child("Local")
                                            )
                                    )
                                    .child(
                                        Button::new("sidebar-new-thread-top")
                                            .ghost()
                                            .small()
                                            .icon(IconName::SquarePen)
                                            .on_click(cx.listener({
                                                let on_new_thread = on_new_thread.clone();
                                                move |this, _, window, cx| on_new_thread(this, window, cx)
                                            }))
                                    )
                            )
                            // Project Scope Selector Card
                            .child(
                                h_flex()
                                    .w_full()
                                    .px_2()
                                    .py_1p5()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().secondary.opacity(0.35))
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
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(cx.theme().foreground)
                                                    .child(project_name)
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(branch_name)
                                    )
                            )
                    )
                    // 2. Action & Filter Bar (Active | Pinned | Settled)
                    .child(
                        h_flex()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .gap_1()
                            .child(
                                Button::new("tab-active")
                                    .when(props.selected_tab == ThreadFilterTab::Active, |b| b.secondary())
                                    .when(props.selected_tab != ThreadFilterTab::Active, |b| b.ghost())
                                    .small()
                                    .flex_1()
                                    .label(format!("Active ({})", active_threads.len() + pinned_threads.len()))
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
                                    .label(format!("Pinned ({})", pinned_threads.len()))
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
                                    .label(format!("Settled ({})", settled_threads.len()))
                                    .on_click(cx.listener({
                                        let on_change_tab = on_change_tab.clone();
                                        move |this, _, window, cx| on_change_tab(this, ThreadFilterTab::Archived, window, cx)
                                    }))
                            )
                    )
                    // 3. Thread Cards Stream
                    .child(
                        v_flex()
                            .p_2()
                            .gap_1p5()
                            .when(is_current_tab_empty, |this| {
                                this.child(
                                    div()
                                        .p_6()
                                        .text_center()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("No threads in this section.")
                                )
                            })
                            .children(thread_card_elements)
                    )
            )
            // 4. Sidebar Bottom Chrome / Footer
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
                                        div()
                                            .size_2()
                                            .rounded_full()
                                            .bg(cx.theme().primary)
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(cx.theme().foreground)
                                            .child(props.active_provider_name.to_string())
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
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} CLI providers active", props.detected_providers_count))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(cx.theme().primary)
                                    .child("Rust · ~89 MB")
                            )
                    )
            )
            .into_any_element()
    }

    fn render_thread_card<V: 'static>(
        thread: &Thread,
        is_active: bool,
        cx: &mut Context<V>,
        on_select_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_pin: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_toggle_archive: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
        on_delete_thread: impl Fn(&mut V, String, &mut Window, &mut Context<V>) + 'static + Clone,
    ) -> impl IntoElement {
        let tid = thread.id.clone();
        let tid_pin = thread.id.clone();
        let tid_arc = thread.id.clone();
        let tid_del = thread.id.clone();
        let is_running = thread.is_running();
        let turns_count = thread.turns.len();

        v_flex()
            .w_full()
            .p_2p5()
            .rounded(cx.theme().radius)
            .gap_1p5()
            .cursor_pointer()
            .border_1()
            .border_color(if is_active {
                cx.theme().primary.opacity(0.4)
            } else {
                cx.theme().border.opacity(0.4)
            })
            .bg(if is_active {
                cx.theme().accent.opacity(0.12)
            } else {
                cx.theme().secondary.opacity(0.15)
            })
            .hover(|s| s.bg(cx.theme().secondary.opacity(0.35)))
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                if is_running {
                                    Icon::new(IconName::RotateCw)
                                        .small()
                                        .text_color(cx.theme().primary)
                                } else if thread.pinned {
                                    Icon::new(IconName::Pin)
                                        .small()
                                        .text_color(cx.theme().primary)
                                } else {
                                    Icon::new(IconName::SquarePen)
                                        .small()
                                        .text_color(cx.theme().muted_foreground)
                                }
                            )
                            .child(
                                div()
                                    .text_sm()
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
                        if is_running {
                            Badge::new().small().child("Running")
                        } else {
                            Badge::new().small().child(format!("{} turns", turns_count))
                        }
                    )
            )
            // Second row: model info & action buttons
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_1()
                            .child(Icon::new(IconName::GitBranch).small().text_color(cx.theme().muted_foreground))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(thread.model.clone())
                            )
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_0p5()
                            // Pin toggle
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
                            // Settle / Archive toggle
                            .child(
                                Button::new(format!("arc-{}", tid_arc))
                                    .ghost()
                                    .small()
                                    .icon(if thread.archived { IconName::FolderPlus } else { IconName::Archive })
                                    .on_click(cx.listener({
                                        let on_toggle_archive = on_toggle_archive.clone();
                                        move |this, _, window, cx| on_toggle_archive(this, tid_arc.clone(), window, cx)
                                    }))
                            )
                            // Delete
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
            )
            .on_mouse_down(gpui::MouseButton::Left, cx.listener({
                let on_select_thread = on_select_thread.clone();
                move |this, _, window, cx| on_select_thread(this, tid.clone(), window, cx)
            }))
    }
}
