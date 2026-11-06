//! Semantic design tokens; replacing the palette requires a rebuild.
#![cfg_attr(
    not(debug_assertions),
    allow(dead_code, reason = "tokens for controls awaiting page migration")
)]
pub(crate) mod controls;
mod palette;
#[allow(dead_code, reason = "complete Penpot token catalog")]
pub(crate) mod tokens {
    include!(concat!(env!("OUT_DIR"), "/design_tokens.rs"));
}

pub(crate) fn register_fonts(cx: &gpui::App) {
    cx.text_system()
        .add_fonts(vec![
            std::borrow::Cow::Borrowed(
                include_bytes!("../../../resources/fonts/AetherisMontserrat-Regular.ttf")
                    .as_slice(),
            ),
            std::borrow::Cow::Borrowed(
                include_bytes!("../../../resources/fonts/AetherisMontserrat-Medium.ttf").as_slice(),
            ),
            std::borrow::Cow::Borrowed(
                include_bytes!("../../../resources/fonts/AetherisMontserrat-Bold.ttf").as_slice(),
            ),
        ])
        .expect("embedded Penpot Montserrat fonts must be valid");
}

pub(crate) const BACKGROUND: u32 = palette::GRAY_950;
pub(crate) const PANEL: u32 = palette::GRAY_900;
pub(crate) const PANEL_MUTED: u32 = palette::GRAY_850;
pub(crate) const BORDER: u32 = palette::GRAY_700;
pub(crate) const TEXT: u32 = palette::GRAY_100;
pub(crate) const TEXT_MUTED: u32 = palette::GRAY_400;
pub(crate) const ACCENT: u32 = palette::TURQUOISE;
pub(crate) const SUCCESS: u32 = palette::GREEN;
pub(crate) const WARNING: u32 = palette::YELLOW_LIGHT;
pub(crate) const DANGER: u32 = palette::RED;
pub(crate) const SELECTION: u32 = palette::BLUE_GRAY;
pub(crate) const TIMER_TEXT: u32 = palette::YELLOW_LIGHT;
pub(crate) const TIMER_UNIT: u32 = palette::YELLOW_DARK;
pub(crate) const SPACE: f32 = tokens::TOKEN_COMPONENTS_COMMON_SPACE;
pub(crate) const RADIUS: f32 = tokens::TOKEN_COMPONENTS_COMMON_RADIUS;
pub(crate) const BORDER_WIDTH: f32 = tokens::TOKEN_COMPONENTS_COMMON_BORDER;
pub(crate) const FONT_SIZE: f32 = tokens::TOKEN_COMPONENTS_COMMON_FONT_SIZE;
pub(crate) const OVERLAY_MARGIN: f32 = tokens::TOKEN_COMPONENTS_LAYOUT_OVERLAY_MARGIN;

#[derive(Clone, Copy, Default)]
pub(crate) enum ControlSize {
    Small,
    #[default]
    Medium,
    Large,
}
impl ControlSize {
    pub(crate) fn height(self) -> f32 {
        match self {
            Self::Small => controls::button::SMALL_HEIGHT,
            Self::Medium => tokens::TOKEN_COMPONENTS_FIELD_HEIGHT,
            Self::Large => controls::button::LARGE_HEIGHT,
        }
    }
}

// Common component dimensions. Page-specific layout constraints stay in pages.
pub(crate) const TABLE_BODY_HEIGHT: f32 = tokens::TOKEN_COMPONENTS_LAYOUT_TABLE_HEIGHT;
pub(crate) const SWITCH_WIDTH: f32 = tokens::TOKEN_COMPONENTS_SWITCH_WIDTH;
pub(crate) const SWITCH_HEIGHT: f32 = tokens::TOKEN_COMPONENTS_SWITCH_HEIGHT;
pub(crate) const CONTROL_HANDLE: f32 = tokens::TOKEN_COMPONENTS_SLIDER_HANDLE;
pub(crate) const SWITCH_PADDING: f32 = tokens::TOKEN_COMPONENTS_SWITCH_PADDING;
pub(crate) const SLIDER_HEIGHT: f32 = tokens::TOKEN_COMPONENTS_SLIDER_HEIGHT;
pub(crate) const SLIDER_TRACK: f32 = tokens::TOKEN_COMPONENTS_SLIDER_TRACK;
pub(crate) const OPTION_MIN_WIDTH: f32 = tokens::TOKEN_COMPONENTS_SEGMENT_MIN_WIDTH;
pub(crate) const OVERLAY_WIDTH: f32 = tokens::TOKEN_COMPONENTS_LAYOUT_OVERLAY_WIDTH;
pub(crate) const OVERLAY_HEIGHT: f32 = tokens::TOKEN_COMPONENTS_LAYOUT_OVERLAY_HEIGHT;
pub(crate) const FONT_FAMILY: &str = tokens::TOKEN_SANS;
pub(crate) const SCRIM: u32 = palette::SCRIM;
