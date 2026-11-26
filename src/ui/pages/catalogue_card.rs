//! Shared Penpot card shell for item, skill and effect presentations.
use crate::ui::{
    components::ScrollContainer,
    theme::{self, tokens as t},
};
use gpui::{
    AnyElement, Div, Image, ImageFormat, IntoElement, div, img, linear_color_stop, linear_gradient,
    prelude::*, px, rgb,
};
use std::sync::Arc;

pub(super) fn header(unique: bool) -> Div {
    div()
        .relative()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .min_h(px(t::TOKEN_COMPONENTS_ITEM_CARD_HEADER_HEIGHT))
        .p(px(12.))
        .rounded_t(px(t::TOKEN_ROUNDED_MD))
        .overflow_hidden()
        .bg(if unique {
            linear_gradient(
                180.,
                linear_color_stop(rgb(t::TOKEN_COMPONENTS_ITEM_CARD_UNIQUE_TOP), 0.),
                linear_color_stop(rgb(t::TOKEN_COMPONENTS_ITEM_CARD_UNIQUE_BOTTOM), 1.),
            )
        } else {
            linear_gradient(
                180.,
                linear_color_stop(rgb(theme::PANEL_MUTED), 0.),
                linear_color_stop(rgb(theme::PANEL), 1.),
            )
        })
}
pub(super) fn card(header: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .w(px(t::TOKEN_COMPONENTS_ITEM_CARD_WIDTH))
        .max_w_full()
        .flex_shrink_0()
        .p(px(2.))
        .gap(px(2.))
        .rounded(px(t::TOKEN_ROUNDED_MD))
        .bg(rgb(theme::PANEL_MUTED))
        .text_color(rgb(theme::TEXT))
        .text_size(px(t::TOKEN_XS))
        .line_height(px(20.))
        .child(header)
}
pub(super) fn section(title: Option<&'static str>) -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .flex_shrink_0()
        .p(px(8.))
        .bg(rgb(theme::PANEL))
        .when_some(title, |el, title| {
            el.child(
                div()
                    .mb(px(4.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(theme::TEXT_MUTED))
                    .child(title),
            )
        })
}
pub(super) fn image(bytes: Option<&[u8]>, mime: Option<&str>) -> Option<Arc<Image>> {
    let format = match mime {
        Some("image/png") => ImageFormat::Png,
        Some("image/jpeg") => ImageFormat::Jpeg,
        Some("image/webp") => ImageFormat::Webp,
        _ => return None,
    };
    bytes.map(|b| Arc::new(Image::from_bytes(format, b.to_vec())))
}
pub(super) fn icon(image: Option<&Arc<Image>>, size: f32) -> Div {
    div()
        .size(px(size))
        .flex_shrink_0()
        .bg(rgb(theme::PANEL_MUTED))
        .overflow_hidden()
        .when_some(image, |el, image| el.child(img(image.clone()).size_full()))
        .when(image.is_none(), |el| {
            el.flex()
                .items_center()
                .justify_center()
                .text_color(rgb(theme::TEXT_MUTED))
                .child("—")
        })
}
pub(super) fn backdrop(image: &Arc<Image>) -> AnyElement {
    img(image.clone())
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .object_fit(gpui::ObjectFit::Fill)
        .into_any_element()
}
pub(super) fn item_icon(
    image: Option<&Arc<Image>>,
    background: Option<&Arc<Image>>,
    size: f32,
) -> Div {
    div()
        .relative()
        .size(px(size))
        .flex_shrink_0()
        .overflow_hidden()
        .when_some(background, |el, bg| {
            el.bg(rgb(theme::PANEL_MUTED)).child(backdrop(bg))
        })
        .child(icon(image, size).relative().bg(gpui::rgba(0x00000000)))
}
pub(super) fn scroll(content: impl IntoElement) -> AnyElement {
    ScrollContainer::new(
        "catalogue-card-scroll",
        div()
            .flex()
            .flex_col()
            .items_center()
            .w_full()
            .p(px(16.))
            .gap(px(12.))
            .child(content),
    )
    .flex_1()
    .min_w_0()
    .min_h_0()
    .into_any_element()
}
