// The pilot page uses a subset of the API; the debug gallery exercises the rest.
#![cfg_attr(
    not(debug_assertions),
    allow(
        dead_code,
        unused_imports,
        reason = "progressive Design System adoption"
    )
)]

mod button;
mod card;
pub(crate) mod icons;
pub(crate) mod state;
pub(crate) mod text_input;
pub(crate) use button::{Button, ButtonVariant, NavigationButton};
pub(crate) use card::{Card, divider};
mod selection;
pub(crate) use selection::{SelectionChanged, Selector, SelectorKind};
mod switch;
pub(crate) use switch::{Switch, SwitchChanged, SwitchVariant};
mod overlay;
mod slider;
pub(crate) use overlay::{
    CloseReason, Overlay, OverlayKind, Placement, dismiss_all, popup_surface,
};
pub(crate) use slider::{InteractionEnded, Slider, SliderChanged};
mod shortcut;
mod timer;
pub(crate) use shortcut::{Shortcut, ShortcutChanged, ShortcutSelector};
pub(crate) use timer::{Deadline, Expired, Timer};
mod number_input;
mod search;
mod table;
pub(crate) use number_input::NumberInput;
pub(crate) use search::{SearchBar, SearchRequested};
pub(crate) use table::{Column, ColumnAlign, ColumnWidth, SortDirection, SortRequest, Table};
mod segment_group;
pub(crate) use segment_group::SegmentGroup;

mod scrollbar;
pub(crate) use scrollbar::ScrollArea;

mod virtual_list;
pub(crate) use virtual_list::{VirtualList, VirtualRows};
