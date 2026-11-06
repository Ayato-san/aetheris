use super::text_input::{TextInput, Validated};
use gpui::{Context, Entity, EventEmitter, Render, Subscription, Window, div, prelude::*};

pub(crate) struct SearchRequested(pub String);
pub(crate) struct SearchBar {
    input: Entity<TextInput>,
    last_sent: Option<String>,
    _subscriptions: Vec<Subscription>,
}
impl EventEmitter<SearchRequested> for SearchBar {}
impl SearchBar {
    pub fn new(placeholder: &str, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::text(placeholder, cx));
        let subscriptions = vec![cx.subscribe(&input, |this, _, _: &Validated, cx| {
            this.search(cx);
        })];
        Self {
            input,
            last_sent: None,
            _subscriptions: subscriptions,
        }
    }
    fn search(&mut self, cx: &mut Context<Self>) {
        let query = self.input.read(cx).content.to_string();
        if self.last_sent.as_ref() != Some(&query) {
            self.last_sent = Some(query.clone());
            cx.emit(SearchRequested(query));
        }
    }
    pub fn set_value(&mut self, value: String, cx: &mut Context<Self>) {
        self.last_sent = Some(value.clone());
        self.input
            .update(cx, |input, cx| input.reset(value, cx).expect("free text"));
    }
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.input
            .update(cx, |input, cx| input.set_disabled(disabled, cx));
    }
}
impl Render for SearchBar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .child(div().flex_1().child(self.input.clone()))
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod interaction_tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use std::{cell::RefCell, rc::Rc, time::Duration};
    #[gpui::test]
    fn search_waits_for_validation_regardless_of_elapsed_time(cx: &mut TestAppContext) {
        cx.update(TextInput::register);
        let (search, cx) = cx.add_window_view(|_, cx| SearchBar::new("Recherche", cx));
        cx.update(|window, cx| window.focus(&search.read(cx).input.read(cx).focus_handle(cx), cx));
        let queries = Rc::new(RefCell::new(Vec::new()));
        let output = queries.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&search, move |_, e: &SearchRequested, _| {
                output.borrow_mut().push(e.0.clone())
            })
        });
        cx.simulate_input("quête");
        cx.executor().advance_clock(Duration::from_secs(30));
        cx.run_until_parked();
        assert!(queries.borrow().is_empty());
        cx.simulate_keystrokes("enter");
        assert_eq!(*queries.borrow(), vec!["quête"]);
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        assert_eq!(queries.borrow().len(), 1);
        cx.update(|_, cx| search.update(cx, |search, cx| search.set_value("programmé".into(), cx)));
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        assert_eq!(queries.borrow().len(), 1);
        cx.simulate_input(" suite");
        cx.executor().advance_clock(Duration::from_secs(30));
        cx.run_until_parked();
        assert_eq!(queries.borrow().len(), 1);
        cx.simulate_keystrokes("enter");
        assert_eq!(
            queries.borrow().last().map(String::as_str),
            Some("programmé suite")
        );
    }
}
