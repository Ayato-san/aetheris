//! Selected sprite preview with stable image identity and virtualized aliases.
use super::catalogue_card;
use crate::ui::{
    components::VirtualList,
    theme::{PANEL, PANEL_MUTED, TEXT_MUTED},
};
use aetheris::game_data::debug_browser::SpriteCard;
use gpui::{AnyElement, App, Image, div, img, prelude::*, px, rgb};
use std::{rc::Rc, sync::Arc};

pub(crate) struct SpritePresentation {
    data: Rc<SpriteCard>,
    image: Option<Arc<Image>>,
    viewport: VirtualList,
}
impl SpritePresentation {
    pub fn can_download(&self) -> bool {
        self.image.is_some() && self.data.image.is_some()
    }
    pub fn download(&self) -> Option<(String, Vec<u8>)> {
        let extension = match self.data.mime.as_deref()? {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            _ => return None,
        };
        let stem: String = self
            .data
            .asset_path
            .rsplit('/')
            .next()?
            .split('.')
            .next()?
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        Some((format!("{stem}.{extension}"), self.data.image.clone()?))
    }
    pub fn new(data: SpriteCard, cx: &App) -> Self {
        Self {
            image: catalogue_card::image(data.image.as_deref(), data.mime.as_deref()),
            viewport: VirtualList::new(data.aliases.len() + 1, cx).gap(8.),
            data: Rc::new(data),
        }
    }
    pub fn render(&self) -> AnyElement {
        let data = self.data.clone();
        let image = self.image.clone();
        self.viewport
            .element("catalogue-sprite", move |index, _, _| {
                if index > 0 {
                    let (table, key) = &data.aliases[index - 1];
                    return div()
                        .p_3()
                        .bg(rgb(PANEL))
                        .text_sm()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(TEXT_MUTED))
                                .child(table.clone()),
                        )
                        .child(key.clone())
                        .into_any_element();
                }
                let scale = (400. / data.width as f32)
                    .min(320. / data.height as f32)
                    .min(4.);
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .child(div().text_sm().child(data.asset_path.clone()))
                    .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child(format!(
                        "#{} · {} × {} px · {}",
                        data.id, data.width, data.height, data.image_state
                    )))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .p_4()
                            .bg(rgb(PANEL_MUTED))
                            .when_some(image.as_ref(), |el, image| {
                                el.child(
                                    img(image.clone())
                                        .w(px(data.width as f32 * scale))
                                        .h(px(data.height as f32 * scale)),
                                )
                            })
                            .when(image.is_none(), |el| el.child("Texture indisponible")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(format!("Références UI ({})", data.aliases.len())),
                    )
                    .into_any_element()
            })
            .min_w_0()
            .min_h_0()
            .into_any_element()
    }
}
