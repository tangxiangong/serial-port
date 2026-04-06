use std::path::PathBuf;

use gpui::{AnyElement, Div, IntoElement, ParentElement, Styled, div, img, px, rgb};

pub(crate) const SIDEBAR_BG: u32 = 0xFBFBFC;
pub(crate) const APP_BG: u32 = 0xFCFCFD;
pub(crate) const PANEL_BG: u32 = 0xFFFFFF;
pub(crate) const PANEL_SOFT: u32 = 0xFAFAFA;
pub(crate) const BORDER: u32 = 0xE5E7EB;
pub(crate) const TEXT_MUTED: u32 = 0x6B7280;
pub(crate) const TEXT_MAIN: u32 = 0x111827;
pub(crate) const CHIP_BLUE: u32 = 0xE0F2FE;
pub(crate) const CHIP_BLUE_TEXT: u32 = 0x0284C7;
pub(crate) const CHIP_DARK: u32 = 0x171717;
pub(crate) const TAB_BG: u32 = 0xF1F1F1;
pub(crate) const CONTROL_ICON_BG: u32 = 0xF3F4F6;

pub(crate) fn asset(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(name)
}

pub(crate) fn svg_icon(name: &str, size: f32) -> AnyElement {
    img(asset(name)).w(px(size)).h(px(size)).into_any_element()
}

pub(crate) fn splitter_handle() -> Div {
    div()
        .w(px(18.))
        .h(px(18.))
        .rounded_md()
        .bg(rgb(0xECECEC))
        .border_1()
        .border_color(rgb(0xD8D8D8))
        .text_xs()
        .text_color(rgb(TEXT_MUTED))
        .items_center()
        .justify_center()
        .child("⋮")
}
