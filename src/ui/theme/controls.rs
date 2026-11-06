//! Button and SegmentGroup metrics and semantic colors from the Penpot export.
use super::tokens;
use gpui::FontWeight;

pub(crate) const FONT_FAMILY: &str = tokens::TOKEN_SANS;
pub(crate) const FONT_SIZE: f32 = tokens::TOKEN_COMPONENTS_COMMON_FONT_SIZE;
pub(crate) const LINE_HEIGHT: f32 = tokens::TOKEN_COMPONENTS_COMMON_LINE_HEIGHT;

pub(crate) mod button {
    use super::{FontWeight, tokens};
    pub const HEIGHT: f32 = tokens::TOKEN_COMPONENTS_BUTTON_HEIGHT;
    pub const SMALL_HEIGHT: f32 = tokens::TOKEN_COMPONENTS_BUTTON_SMALL_HEIGHT;
    pub const LARGE_HEIGHT: f32 = tokens::TOKEN_COMPONENTS_BUTTON_LARGE_HEIGHT;
    pub const MIN_WIDTH: f32 = tokens::TOKEN_COMPONENTS_BUTTON_MIN_WIDTH;
    pub const PADDING_X: f32 = tokens::TOKEN_COMPONENTS_BUTTON_PADDING_X;
    pub const RADIUS: f32 = tokens::TOKEN_COMPONENTS_BUTTON_RADIUS;
    pub const BORDER_WIDTH: f32 = tokens::TOKEN_COMPONENTS_BUTTON_BORDER;
    pub const WEIGHT: FontWeight = FontWeight(tokens::TOKEN_COMPONENTS_BUTTON_WEIGHT);
    pub const TEXT: u32 = tokens::TOKEN_COMPONENTS_BUTTON_TEXT;
    pub const PRIMARY_BACKGROUND: u32 = tokens::TOKEN_COMPONENTS_BUTTON_PRIMARY_BACKGROUND;
    pub const PRIMARY_HOVER: u32 = tokens::TOKEN_COMPONENTS_BUTTON_PRIMARY_HOVER;
    pub const PRIMARY_BORDER: u32 = tokens::TOKEN_COMPONENTS_BUTTON_PRIMARY_BORDER;
    pub const SECONDARY_BACKGROUND: u32 = tokens::TOKEN_COMPONENTS_BUTTON_SECONDARY_BACKGROUND;
    pub const SECONDARY_HOVER: u32 = tokens::TOKEN_COMPONENTS_BUTTON_SECONDARY_HOVER;
    pub const SECONDARY_BORDER: u32 = tokens::TOKEN_COMPONENTS_BUTTON_SECONDARY_BORDER;
}

pub(crate) mod segment_group {
    use super::{FontWeight, tokens};
    pub const WIDTH: f32 = tokens::TOKEN_COMPONENTS_SEGMENT_WIDTH;
    pub const HEIGHT: f32 = tokens::TOKEN_COMPONENTS_SEGMENT_HEIGHT;
    pub const PADDING: f32 = tokens::TOKEN_COMPONENTS_SEGMENT_PADDING;
    pub const PILL_HEIGHT: f32 = HEIGHT - PADDING * 2.;
    pub const WEIGHT: FontWeight = FontWeight(tokens::TOKEN_COMPONENTS_SEGMENT_WEIGHT);
    pub const BACKGROUND: u32 = tokens::TOKEN_COMPONENTS_SEGMENT_BACKGROUND;
    pub const TEXT: u32 = tokens::TOKEN_COMPONENTS_SEGMENT_TEXT;
    pub const HOVER_BACKGROUND: u32 = tokens::TOKEN_COMPONENTS_SEGMENT_HOVER_BACKGROUND;
    pub const HOVER_TEXT: u32 = tokens::TOKEN_COMPONENTS_SEGMENT_HOVER_TEXT;
    pub const ACTIVE_BACKGROUND: u32 = tokens::TOKEN_COMPONENTS_SEGMENT_ACTIVE_BACKGROUND;
    pub const ACTIVE_TEXT: u32 = tokens::TOKEN_COMPONENTS_SEGMENT_ACTIVE_TEXT;
    pub const DISABLED_TEXT: u32 = tokens::TOKEN_COMPONENTS_SEGMENT_DISABLED_TEXT;
}
