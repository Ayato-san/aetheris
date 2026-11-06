//! Single-line input using GPUI’s native text-input and selection protocol.
// Adapted from GPUI's examples/input.rs (Zed Industries, Apache-2.0).
// Source: https://github.com/zed-industries/zed/blob/1f32ec0c/crates/gpui/examples/input.rs
// Changes: constructor, scoped bindings, character boundaries, single-line paste,
// focus handling and Aetheris styling. License: LICENSES/GPUI-Apache-2.0.txt.
use super::state::{FieldState, NumberConfig, Validation};
use crate::ui::theme::{self, tokens as t};

type TextAcceptance = dyn Fn(&str, &mut App) -> Result<String, String>;

pub(crate) use super::state::NumericSyntax;

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, GlobalElementId, KeyBinding,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, SharedString, Style, TextRun, UTF16Selection, UnderlineStyle, Window, actions, div,
    fill, point, prelude::*, px, relative, rgb, size,
};

actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        Quit,
    ]
);

pub(crate) struct ContentChanged;

impl EventEmitter<ContentChanged> for TextInput {}
pub(crate) struct Validated;
impl EventEmitter<Validated> for TextInput {}
pub(crate) struct ValueChanged;
impl EventEmitter<ValueChanged> for TextInput {}
pub(crate) struct DraftCancelled;
impl EventEmitter<DraftCancelled> for TextInput {}

pub(crate) struct TextInput {
    read_only: bool,
    numeric: Option<NumericSyntax>,
    focus_handle: FocusHandle,
    pub(crate) content: SharedString,
    placeholder: SharedString,
    label: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    disabled: bool,
    field: FieldState,
    number_config: Option<NumberConfig>,
    on_change_requested: Option<Box<TextAcceptance>>,
    blur: Option<gpui::Subscription>,
}

impl TextInput {
    pub(crate) fn new(
        value: String,
        placeholder: String,
        numeric: NumericSyntax,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            disabled: false,
            field: FieldState::new(value.clone()),
            number_config: Some(NumberConfig {
                decimals: numeric.decimal,
                negative: numeric.signed,
                allow_empty: true,
                ..Default::default()
            }),
            on_change_requested: None,
            blur: None,
            read_only: false,
            numeric: Some(numeric),
            focus_handle: cx.focus_handle().tab_stop(true),
            content: value.into(),
            label: placeholder.clone().into(),
            placeholder: placeholder.into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        }
    }
    pub(crate) fn text(placeholder: &str, cx: &mut Context<Self>) -> Self {
        let mut input = Self::new(
            String::new(),
            placeholder.into(),
            NumericSyntax {
                decimal: false,
                signed: false,
            },
            cx,
        );
        input.numeric = None;
        input.number_config = None;
        input
    }
    pub(crate) fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.label = label.into();
        cx.notify();
    }
    pub(crate) fn set_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        self.read_only = read_only;
        self.marked_range = None;
        cx.notify();
    }

    pub(crate) fn on_change_requested(
        &mut self,
        callback: impl Fn(&str, &mut App) -> Result<String, String> + 'static,
    ) {
        self.on_change_requested = Some(Box::new(callback));
    }

    pub(crate) fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(!disabled);
        self.is_selecting = false;
        cx.notify();
    }
    /// Replace numeric configuration and reference atomically, without events.
    pub(crate) fn configure_number(
        &mut self,
        config: NumberConfig,
        value: String,
        cx: &mut Context<Self>,
    ) -> Validation<()> {
        config.parse(&value)?;
        let changed = self.number_config != Some(config);
        self.numeric = Some(NumericSyntax {
            decimal: config.decimals,
            signed: config.negative,
        });
        self.number_config = Some(config);
        self.set_content(value, cx)?;
        if changed && self.field.dirty() {
            self.field.conflict = true;
            cx.notify();
        }
        Ok(())
    }
    /// Synchronization preserves an active draft; explicit reset abandons it.
    pub(crate) fn set_content(&mut self, value: String, cx: &mut Context<Self>) -> Validation<()> {
        let value = match self.number_config {
            Some(config) => config.normalize(&value)?,
            None => value,
        };
        if self.field.reference == value {
            return Ok(());
        }
        self.field.draft = self.content.to_string();
        let editing = self.field.dirty() || self.marked_range.is_some();
        self.field.sync(value);
        self.content = self.field.draft.clone().into();
        if !editing {
            self.clamp_selection();
        }
        cx.notify();
        Ok(())
    }
    pub(crate) fn reset(&mut self, value: String, cx: &mut Context<Self>) -> Validation<()> {
        let value = match self.number_config {
            Some(config) => config.normalize(&value)?,
            None => value,
        };
        self.field.reset(value.clone());
        self.content = value.into();
        self.selected_range = self.content.len()..self.content.len();
        self.marked_range = None;
        cx.notify();
        Ok(())
    }
    pub(crate) fn reset_reference(&mut self, cx: &mut Context<Self>) {
        self.reset(self.field.reference.clone(), cx)
            .expect("validated reference");
    }
    fn clamp_selection(&mut self) {
        // A replaced reference may be shorter or have different UTF-8 boundaries.
        self.selected_range = self.content.len()..self.content.len();
        self.marked_range = None;
    }
    pub(crate) fn accept(&mut self, cx: &mut Context<Self>) {
        if self.read_only || self.disabled || self.marked_range.is_some() {
            return;
        }
        let candidate = self.content.to_string();
        self.field.draft = candidate.clone();
        if !self.field.dirty() && self.field.error.is_none() {
            cx.emit(Validated);
            return;
        }
        let result = if let Some(config) = self.number_config {
            config.normalize(&candidate).map_err(|e| e.0)
        } else {
            Ok(candidate)
        };
        let result = result
            .and_then(|candidate| {
                if candidate == self.field.reference {
                    return Ok(candidate);
                }
                match &self.on_change_requested {
                    Some(callback) => callback(&candidate, cx),
                    None => Ok(candidate),
                }
            })
            .and_then(|accepted| match self.number_config {
                Some(config) => config.normalize(&accepted).map_err(|error| error.0),
                None => Ok(accepted),
            });
        match result {
            Ok(value) => {
                let changed = self.field.accept(value.clone());
                self.content = value.into();
                self.clamp_selection();
                if changed {
                    cx.emit(ValueChanged);
                }
                cx.emit(Validated);
            }
            Err(error) => self.field.error = Some(error),
        }
        cx.notify();
    }
    pub(crate) fn register(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, Some("TextInput")),
            KeyBinding::new("delete", Delete, Some("TextInput")),
            KeyBinding::new("left", Left, Some("TextInput")),
            KeyBinding::new("right", Right, Some("TextInput")),
            KeyBinding::new("shift-left", SelectLeft, Some("TextInput")),
            KeyBinding::new("shift-right", SelectRight, Some("TextInput")),
            KeyBinding::new("ctrl-a", SelectAll, Some("TextInput")),
            KeyBinding::new("ctrl-v", Paste, Some("TextInput")),
            KeyBinding::new("ctrl-c", Copy, Some("TextInput")),
            KeyBinding::new("ctrl-x", Cut, Some("TextInput")),
            KeyBinding::new("home", Home, Some("TextInput")),
            KeyBinding::new("end", End, Some("TextInput")),
        ]);
    }

    fn left(&mut self, _: &Left, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), window, cx);
        } else {
            self.move_to(self.selected_range.start, window, cx)
        }
    }

    fn right(&mut self, _: &Right, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), window, cx);
        } else {
            self.move_to(self.selected_range.end, window, cx)
        }
    }

    fn select_left(&mut self, _: &SelectLeft, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), window, cx);
    }

    fn select_right(&mut self, _: &SelectRight, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), window, cx);
    }

    fn select_all(&mut self, _: &SelectAll, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx)
    }

    fn home(&mut self, _: &Home, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    fn end(&mut self, _: &End, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                window.play_system_bell();
                return;
            }
            self.select_to(prev, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                window.play_system_bell();
                return;
            }
            self.select_to(next, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        window.focus(&self.focus_handle, cx);
        self.is_selecting = true;
        window.request_virtual_keyboard();

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), window, cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        }
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace("\n", " "), window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }
    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    fn select_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = self.range_from_utf16(&range_utf16);
        self.selection_reversed = false;
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    fn text_length_utf16(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.content.encode_utf16().count())
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.read_only || self.disabled {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let candidate =
            self.content[0..range.start].to_owned() + new_text + &self.content[range.end..];
        if self
            .numeric
            .is_some_and(|numeric| !numeric.accepts(&candidate))
        {
            window.play_system_bell();
            return;
        }
        let changed = self.content.as_ref() != candidate;
        self.content = candidate.into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
        window.notify_text_input_changed(&self.focus_handle);
        if changed {
            self.field.draft = self.content.to_string();
            cx.emit(ContentChanged);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.read_only || self.disabled {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let candidate =
            self.content[0..range.start].to_owned() + new_text + &self.content[range.end..];
        let changed = self.content.as_ref() != candidate;
        self.content = candidate.into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        let insertion_start_utf16 = self.offset_to_utf16(range.start);
        let inserted_length_utf16 = new_text.encode_utf16().count();
        self.selected_range = new_selected_range_utf16
            .map(|selected| {
                self.range_from_utf16(
                    &(insertion_start_utf16 + selected.start.min(inserted_length_utf16)
                        ..insertion_start_utf16 + selected.end.min(inserted_length_utf16)),
                )
            })
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.selection_reversed = false;

        window.notify_text_input_changed(&self.focus_handle);
        if changed {
            self.field.draft = self.content.to_string();
            cx.emit(ContentChanged);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        if last_layout.text != self.content {
            return None;
        }
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn selection_bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Bounds<Pixels>> {
        self.bounds_for_range(range_utf16, bounds, window, cx)
            .into_iter()
            .collect()
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;

        if last_layout.text != self.content {
            return None;
        }
        let utf8_index = last_layout.closest_index_for_x(line_point.x);
        Some(self.offset_to_utf16(utf8_index))
    }
}

struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (
                input.placeholder.clone(),
                rgb(t::TOKEN_COMPONENTS_FIELD_PLACEHOLDER).into(),
            )
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(2.), bounds.bottom() - bounds.top()),
                    ),
                    rgb(theme::ACCENT),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgb(theme::SELECTION),
                )),
                None,
            )
        };
        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection)
        }
        let line = prepaint.line.take().unwrap();
        line.paint(
            bounds.origin,
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        )
        .unwrap();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.blur.is_none() {
            self.blur =
                Some(cx.on_blur(&self.focus_handle, _window, |input, _, cx| input.accept(cx)));
        }
        div()
            .id("text-input")
            .role(gpui::Role::TextInput)
            .aria_label(self.label.clone())
            .aria_value(self.content.clone())
            .aria_description(self.field.error.clone().unwrap_or_else(|| {
                if self.field.conflict {
                    "Valeur modifiée ailleurs".into()
                } else if self.read_only {
                    "Lecture seule".into()
                } else {
                    String::new()
                }
            }))
            .when(self.disabled, |d| d.aria_description("Indisponible"))
            .flex()
            .flex_col()
            .tab_stop(!self.disabled)
            .on_key_down(
                cx.listener(|input, event: &gpui::KeyDownEvent, window, cx| {
                    if input.marked_range.is_some() {
                        if ["enter", "escape"].contains(&event.keystroke.key.as_str()) {
                            cx.stop_propagation();
                        }
                        return;
                    }
                    match event.keystroke.key.as_str() {
                        "enter" => {
                            input.accept(cx);
                            window.prevent_default();
                            cx.stop_propagation();
                        }
                        "tab" => {
                            input.accept(cx);
                            if event.keystroke.modifiers.shift {
                                window.focus_prev(cx);
                            } else {
                                window.focus_next(cx);
                            }
                            cx.stop_propagation();
                        }
                        "escape" => {
                            input.field.draft = input.content.to_string();
                            if input.field.cancel() {
                                input.content = input.field.draft.clone().into();
                                input.clamp_selection();
                                cx.emit(DraftCancelled);
                                cx.notify();
                                cx.stop_propagation();
                            }
                        }
                        _ => {}
                    }
                }),
            )
            .key_context("TextInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(if self.read_only {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .when(self.disabled, |d| d.opacity(0.55))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .gap(px(t::TOKEN_COMPONENTS_FIELD_GAP))
            .font_family(theme::FONT_FAMILY)
            .font_weight(gpui::FontWeight(t::TOKEN_COMPONENTS_FIELD_WEIGHT))
            .text_color(rgb(t::TOKEN_COMPONENTS_FIELD_TEXT))
            .line_height(px(t::TOKEN_COMPONENTS_COMMON_LINE_HEIGHT))
            .text_size(px(theme::FONT_SIZE))
            .child(
                div()
                    .h(px(theme::ControlSize::Medium.height()))
                    .w_full()
                    .flex()
                    .items_center()
                    .px(px(t::TOKEN_COMPONENTS_FIELD_PADDING_X))
                    .py(px(t::TOKEN_COMPONENTS_FIELD_PADDING_Y))
                    .rounded(px(theme::RADIUS))
                    .border(px(
                        if self.field.error.is_none() && self.focus_handle.is_focused(_window) {
                            t::TOKEN_COMPONENTS_COMMON_FOCUS_WIDTH
                        } else {
                            theme::BORDER_WIDTH
                        },
                    ))
                    .border_color(rgb(if self.field.error.is_some() {
                        t::TOKEN_COMPONENTS_FIELD_ERROR
                    } else if self.focus_handle.is_focused(_window) {
                        t::TOKEN_COMPONENTS_COMMON_FOCUS_COLOR
                    } else {
                        t::TOKEN_COMPONENTS_FIELD_BORDER
                    }))
                    .bg(rgb(t::TOKEN_COMPONENTS_FIELD_BACKGROUND))
                    .child(TextElement { input: cx.entity() }),
            )
            .when_some(self.field.error.clone(), |d, error| {
                d.child(
                    div()
                        .text_size(px(theme::FONT_SIZE))
                        .font_weight(gpui::FontWeight(t::TOKEN_FONT_MEDIUM))
                        .text_color(rgb(theme::DANGER))
                        .child(error),
                )
            })
            .when(self.field.conflict, |d| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme::TEXT_MUTED))
                        .child("Valeur modifiée ailleurs — Escape pour la reprendre"),
                )
            })
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::TestAppContext;
    use std::{cell::Cell, rc::Rc};

    #[gpui::test]
    fn enter_blur_programmatic_updates_and_rejection(cx: &mut TestAppContext) {
        cx.update(TextInput::register);
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = TextInput::text("Texte", cx);
            window.focus(&input.focus_handle, cx);
            input
        });
        let changes = Rc::new(Cell::new(0));
        let counter = changes.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&input, move |_, _: &ValueChanged, _| {
                counter.set(counter.get() + 1)
            })
        });
        cx.simulate_input("Bonjour");
        assert_eq!(changes.get(), 0);
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| assert_eq!(input.read(cx).field.reference, "Bonjour"));
        cx.update(|window, cx| window.blur(cx));
        cx.run_until_parked();
        assert_eq!(changes.get(), 1);
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                input.set_content("Référence".into(), cx).unwrap()
            });
            let focus = input.read(cx).focus_handle.clone();
            window.focus(&focus, cx);
        });
        assert_eq!(changes.get(), 1);
        cx.simulate_input(" brouillon");
        cx.update(|_, cx| {
            input.update(cx, |input, cx| {
                input.set_content("Nouvelle référence".into(), cx).unwrap()
            })
        });
        cx.update(|_, cx| assert!(input.read(cx).field.conflict));
        cx.simulate_keystrokes("escape");
        cx.update(|_, cx| assert_eq!(input.read(cx).content.as_ref(), "Nouvelle référence"));
        cx.update(|_, cx| {
            input.update(cx, |input, _| {
                input.on_change_requested(|_, _| Err("Refus de la page".into()))
            })
        });
        cx.simulate_input(" refusé");
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| {
            let field = &input.read(cx).field;
            assert_eq!(field.reference, "Nouvelle référence");
            assert_eq!(field.error.as_deref(), Some("Refus de la page"));
            assert!(field.dirty());
        });
        assert_eq!(changes.get(), 1);
    }

    #[gpui::test]
    fn numeric_paste_ime_and_atomic_configuration(cx: &mut TestAppContext) {
        cx.update(TextInput::register);
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = super::super::NumberInput::create(
                "1".into(),
                "Nombre".into(),
                NumberConfig {
                    min: Some(0.),
                    max: Some(10.),
                    decimals: true,
                    ..Default::default()
                },
                cx,
            )
            .unwrap();
            window.focus(&input.focus_handle, cx);
            input
        });
        cx.simulate_keystrokes("ctrl-a");
        cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("12abc".into())));
        cx.simulate_keystrokes("ctrl-v");
        cx.update(|_, cx| assert_eq!(input.read(cx).content.as_ref(), "1"));
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                input.replace_and_mark_text_in_range(Some(0..1), "に", None, window, cx);
                input.accept(cx);
                assert_eq!(input.field.reference, "1");
                input.replace_text_in_range(None, "2,", window, cx);
                input.accept(cx);
                assert_eq!(input.field.reference, "2");
                let invalid = NumberConfig {
                    min: Some(5.),
                    max: Some(3.),
                    ..Default::default()
                };
                assert!(input.configure_number(invalid, "6".into(), cx).is_err());
                assert_eq!(input.field.reference, "2");
                input.set_content("9".into(), cx).unwrap();
                assert_eq!(input.content.as_ref(), "9");
            })
        });
    }

    #[gpui::test]
    fn locking_preserves_drafts_without_committing(cx: &mut TestAppContext) {
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = TextInput::text("Texte", cx);
            window.focus(&input.focus_handle, cx);
            input
        });
        cx.simulate_input("brouillon");
        cx.update(|_, cx| {
            input.update(cx, |input, cx| {
                input.set_read_only(true, cx);
                input.accept(cx);
                assert_eq!(input.field.reference, "");
                assert_eq!(input.content.as_ref(), "brouillon");
                input.set_content("externe".into(), cx).unwrap();
                assert!(input.field.conflict);
                assert_eq!(input.content.as_ref(), "brouillon");
            })
        });
    }
}
