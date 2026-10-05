pub mod chat;
pub mod composer;
pub mod diff_panel;
pub mod header;
pub mod model_picker;
pub mod rename_modal;
pub mod settings_modal;
pub mod sidebar;

use gpui_kit::*;

#[inline]
pub fn h_flex() -> Div {
    div().flex().flex_row()
}

#[inline]
pub fn v_flex() -> Div {
    div().flex().flex_col()
}
