use gpui::{div, prelude::*, rgb};

use crate::ui::theme::{ACCENT, PANEL, PANEL_MUTED, TEXT_MUTED};

pub(crate) fn section(index: usize) -> gpui::AnyElement {
    match index {
        0 => (div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_3xl().child("Aetheris"))
                .child(
                    div()
                        .text_color(rgb(TEXT_MUTED))
                        .child("Votre compagnon de jeu, au même endroit."),
                )).into_any_element(),
        1 => (div()
                .flex()
                .flex_col()
                .gap_3()
                .p_6()
                .rounded_xl()
                .bg(rgb(PANEL))
                .border_1()
                .border_color(rgb(PANEL_MUTED))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(ACCENT))
                        .child("PREMIER MODULE"),
                )
                .child(div().text_2xl().child("Configuration Engine.ini"))
                .child(
                    div()
                        .text_color(rgb(TEXT_MUTED))
                        .child("Chargez, modifiez et sauvegardez les paramètres du jeu sans éditer le fichier à la main."),
                )).into_any_element(),
        2 => (div()
                .flex()
                .gap_4()
                .child(feature_card("Historique", "Donjons réalisés"))
                .child(feature_card("Combat meter", "Analyse en direct"))
                .child(feature_card("Carte", "Familiers et ressources"))
                .child(feature_card("Timers", "Événements à suivre"))).into_any_element(),
        _ => unreachable!("unknown home section"),
    }
}

fn feature_card(title: &'static str, subtitle: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .p_4()
        .rounded_lg()
        .bg(rgb(PANEL))
        .border_1()
        .border_color(rgb(PANEL_MUTED))
        .child(title)
        .child(div().text_sm().text_color(rgb(TEXT_MUTED)).child(subtitle))
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use crate::ui::components::VirtualList;
    use gpui::{Context, Render, TestAppContext, Window, px};
    struct Home {
        viewport: VirtualList,
    }
    impl Render for Home {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.viewport
                .element("home", |index, _, _| section(index))
                .w(px(500.))
                .h(px(180.))
        }
    }
    #[gpui::test]
    fn small_viewports_can_scroll_through_all_home_sections(cx: &mut TestAppContext) {
        cx.update(|cx| crate::ui::theme::register_fonts(cx));
        let (home, cx) = cx.add_window_view(|_, cx| Home {
            viewport: VirtualList::new(3, cx),
        });
        cx.update(|window, cx| {
            let state = home.read(cx).viewport.state();
            assert!(state.max_offset_for_scrollbar().y > px(0.));
            state.scroll_to_reveal_item(2);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.update(|_, cx| assert!(home.read(cx).viewport.state().bounds_for_item(2).is_some()));
    }
}
