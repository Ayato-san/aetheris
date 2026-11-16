//! Shared presentation and capture lifecycle for overlay widgets.
use crate::capture::widgets::{self, Consumer};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Growth {
    #[default]
    TopDown,
    BottomUp,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Privacy {
    #[default]
    Public,
    ExcludeFromCapture,
}

/// Coordinates are pixels relative to the game's client area. `y` anchors the
/// top for TopDown and the bottom for BottomUp. Height is a content-height cap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetConfig {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
    pub enabled: bool,
    pub privacy: Privacy,
    pub growth: Growth,
    /// Scale of content and background; 1 is 100%. Position stays in pixels.
    pub zoom: f32,
    /// Whole-widget opacity, including content, in 0..=1.
    pub opacity: f32,
    /// Background opacity before applying whole-widget opacity, in 0..=1.
    pub background_opacity: f32,
}
impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            x: 20,
            y: 20,
            width: 360,
            height: 600,
            enabled: true,
            privacy: Privacy::Public,
            growth: Growth::TopDown,
            zoom: 1.,
            opacity: 230. / 255.,
            background_opacity: 1.,
        }
    }
}
impl WidgetConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.width == 0 || self.height == 0 || self.width > 4096 || self.height > 4096 {
            return Err("Widget dimensions must be between 1 and 4096 pixels");
        }
        if !self.zoom.is_finite() || !(0.25..=4.).contains(&self.zoom) {
            return Err("Widget zoom must be finite and between 0.25 and 4");
        }
        if f32::from(self.width) * self.zoom > 4096. || f32::from(self.height) * self.zoom > 4096. {
            return Err("Widget dimensions after zoom must not exceed 4096 pixels");
        }
        if !self.opacity.is_finite()
            || !self.background_opacity.is_finite()
            || !(0. ..=1.).contains(&self.opacity)
            || !(0. ..=1.).contains(&self.background_opacity)
        {
            return Err("Widget opacity must be finite and between 0 and 1");
        }
        Ok(())
    }

    pub fn layout(&self, content_height: i32, offset: i32) -> WidgetBounds {
        let height = ((content_height.clamp(1, i32::from(self.height).max(1)) as f32 * self.zoom)
            .round() as i32)
            .max(1);
        let anchor = self
            .y
            .saturating_add((offset as f32 * self.zoom).round() as i32);
        WidgetBounds {
            x: self.x,
            y: match self.growth {
                Growth::TopDown => anchor,
                Growth::BottomUp => anchor.saturating_sub(height),
            },
            width: ((f32::from(self.width) * self.zoom).round() as i32).max(1),
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// A widget owns its content; the renderer consumes owned snapshots without
/// holding capture locks. None means there is currently nothing to display.
pub trait WidgetContent {
    type Frame;
    fn snapshot(&self) -> Option<Self::Frame>;
}

/// A ready-to-use text frame for widgets that do not need custom drawing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextFrame {
    pub text: String,
    pub height: u16,
}
impl WidgetContent for TextFrame {
    type Frame = Self;
    fn snapshot(&self) -> Option<Self> {
        (!self.text.is_empty()).then(|| self.clone())
    }
}

pub struct Widget<C> {
    config: WidgetConfig,
    content: C,
}
impl<C: WidgetContent> Widget<C> {
    pub fn new(config: WidgetConfig, content: C) -> Result<Self, &'static str> {
        config.validate()?;
        Ok(Self { config, content })
    }
    pub fn config(&self) -> WidgetConfig {
        self.config
    }
    pub fn configure(&mut self, config: WidgetConfig) -> Result<(), &'static str> {
        config.validate()?;
        self.config = config;
        Ok(())
    }
    pub fn update_content(&mut self, content: C) {
        self.content = content;
    }
    /// Produce content for the renderer. Platform visibility and capture
    /// protection must still be verified before showing a native window.
    pub fn show(&self) -> Option<C::Frame> {
        self.config
            .enabled
            .then(|| self.content.snapshot())
            .flatten()
    }
}

/// Reusable activation for an opcode consumer. Disabled consumers are removed
/// from the shared pipeline and reset, so reactivation cannot reuse stale data.
pub struct WidgetSubscription {
    consumer: Arc<Mutex<dyn Consumer>>,
    gate: Arc<Mutex<Gate>>,
    subscription: Option<widgets::Subscription>,
}
struct Gate {
    consumer: Arc<Mutex<dyn Consumer>>,
    enabled: bool,
}
impl Consumer for Gate {
    fn accepts(&self, opcode: [u8; 2], entity: Option<u32>) -> bool {
        self.enabled && self.consumer.lock().unwrap().accepts(opcode, entity)
    }
    fn message(&mut self, opcode: [u8; 2], payload: &[u8], now: std::time::Instant) {
        if self.enabled {
            self.consumer.lock().unwrap().message(opcode, payload, now);
        }
    }
    fn reset(&mut self, error: Option<&str>) {
        if self.enabled {
            self.consumer.lock().unwrap().reset(error);
        }
    }
    fn interrupted(&mut self, reason: &str, now: std::time::Instant) {
        if self.enabled {
            self.consumer.lock().unwrap().interrupted(reason, now);
        }
    }
    fn resynchronized(&mut self) {
        if self.enabled {
            self.consumer.lock().unwrap().resynchronized();
        }
    }
}
impl WidgetSubscription {
    pub fn new(consumer: Arc<Mutex<dyn Consumer>>, enabled: bool) -> Self {
        let gate = Arc::new(Mutex::new(Gate {
            consumer: consumer.clone(),
            enabled: false,
        }));
        let mut this = Self {
            consumer,
            gate,
            subscription: None,
        };
        this.set_enabled(enabled);
        this
    }
    pub fn enabled(&self) -> bool {
        self.subscription.is_some()
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled() == enabled {
            return;
        }
        if enabled {
            self.gate.lock().unwrap().enabled = true;
            self.subscription = Some(widgets::subscribe(self.gate.clone()));
        } else {
            // Serialize with any delivery already copied by the hub.
            self.gate.lock().unwrap().enabled = false;
            self.subscription = None;
            self.consumer.lock().unwrap().reset(None);
        }
    }
}
impl Drop for WidgetSubscription {
    fn drop(&mut self) {
        self.set_enabled(false);
    }
}

/// Composite premultiplied GDI content and its white coverage mask over a
/// translucent background. Whole-widget opacity is applied by the compositor.
#[cfg(any(windows, test))]
pub(super) fn composite_pixel(content: [u8; 4], coverage: u8, background_opacity: f32) -> [u8; 4] {
    let background = ((255 - coverage) as f32 * background_opacity).round() as u8;
    let channel = |index: usize, color: u8| {
        (u16::from(content[index]) + u16::from(color) * u16::from(background) / 255).min(255) as u8
    };
    [
        channel(0, 0x18),
        channel(1, 0x1d),
        channel(2, 0x24),
        coverage.saturating_add(background),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_grows_from_the_selected_anchor_and_respects_the_height_cap() {
        let mut config = WidgetConfig {
            y: 400,
            height: 100,
            ..WidgetConfig::default()
        };
        assert_eq!(config.layout(40, 0).y, 400);
        assert_eq!(config.layout(80, 0).y, 400);
        config.growth = Growth::BottomUp;
        assert_eq!(
            config.layout(40, 0),
            WidgetBounds {
                x: 20,
                y: 360,
                width: 360,
                height: 40
            }
        );
        assert_eq!(config.layout(80, 0).y, 320);
        assert_eq!(
            config.layout(200, 10),
            WidgetBounds {
                x: 20,
                y: 310,
                width: 360,
                height: 100
            }
        );
        config.y = i32::MIN;
        assert_eq!(config.layout(100, -1).y, i32::MIN);
    }

    #[test]
    fn content_updates_and_activation_share_the_presentation_lifecycle() {
        let mut widget = Widget::new(
            WidgetConfig::default(),
            TextFrame {
                text: "First".into(),
                height: 20,
            },
        )
        .unwrap();
        assert_eq!(widget.show().unwrap().text, "First");
        widget.update_content(TextFrame {
            text: "Updated\ncontent".into(),
            height: 40,
        });
        assert_eq!(widget.show().unwrap().height, 40);
        widget
            .configure(WidgetConfig {
                enabled: false,
                ..widget.config()
            })
            .unwrap();
        assert!(widget.show().is_none());
        widget
            .configure(WidgetConfig {
                enabled: true,
                ..widget.config()
            })
            .unwrap();
        assert_eq!(widget.show().unwrap().text, "Updated\ncontent");
        widget.update_content(TextFrame {
            text: String::new(),
            height: 20,
        });
        assert!(widget.show().is_none());
    }

    #[test]
    fn invalid_configuration_is_rejected_without_changing_the_widget() {
        let original = WidgetConfig::default();
        let mut widget = Widget::new(
            original,
            TextFrame {
                text: "Text".into(),
                height: 20,
            },
        )
        .unwrap();
        for invalid in [
            WidgetConfig {
                width: 0,
                ..original
            },
            WidgetConfig {
                height: 4097,
                ..original
            },
            WidgetConfig {
                opacity: f32::NAN,
                ..original
            },
            WidgetConfig {
                opacity: -0.1,
                ..original
            },
            WidgetConfig {
                background_opacity: f32::INFINITY,
                ..original
            },
            WidgetConfig {
                background_opacity: 1.1,
                ..original
            },
            WidgetConfig {
                zoom: 0.,
                ..original
            },
            WidgetConfig {
                zoom: f32::NAN,
                ..original
            },
            WidgetConfig {
                zoom: 4.1,
                ..original
            },
            WidgetConfig {
                zoom: 2.,
                width: 3000,
                ..original
            },
        ] {
            assert!(widget.configure(invalid).is_err());
            assert_eq!(widget.config(), original);
        }
    }

    #[test]
    fn zoom_scales_content_and_dimensions_while_preserving_the_anchor() {
        let config = WidgetConfig {
            x: 70,
            y: 400,
            zoom: 2.,
            ..WidgetConfig::default()
        };
        assert_eq!(
            config.layout(40, 0),
            WidgetBounds {
                x: 70,
                y: 400,
                width: 720,
                height: 80
            }
        );
        assert_eq!(
            WidgetConfig {
                growth: Growth::BottomUp,
                ..config
            }
            .layout(40, 0),
            WidgetBounds {
                x: 70,
                y: 320,
                width: 720,
                height: 80
            }
        );
        assert_eq!(
            WidgetConfig {
                zoom: 0.5,
                ..config
            }
            .layout(40, 0),
            WidgetBounds {
                x: 70,
                y: 400,
                width: 180,
                height: 20
            }
        );
    }

    #[test]
    fn background_transparency_preserves_text_and_antialiased_edges() {
        assert_eq!(composite_pixel([0; 4], 0, 0.), [0; 4]);
        assert_eq!(composite_pixel([0; 4], 0, 1.), [0x18, 0x1d, 0x24, 255]);
        assert_eq!(
            composite_pixel([245, 245, 245, 0], 255, 0.),
            [245, 245, 245, 255]
        );
        assert_eq!(
            composite_pixel([245, 245, 245, 0], 255, 1.),
            [245, 245, 245, 255]
        );
        assert_eq!(
            composite_pixel([100, 100, 100, 0], 128, 0.),
            [100, 100, 100, 128]
        );
        assert_eq!(composite_pixel([100, 100, 100, 0], 128, 1.)[3], 255);
        assert_eq!(composite_pixel([0; 4], 0, 0.5)[3], 128);
    }

    #[derive(Default)]
    struct Counter(usize);
    impl Consumer for Counter {
        fn accepts(&self, opcode: [u8; 2], _: Option<u32>) -> bool {
            opcode == [0x34, 0x36]
        }
        fn message(&mut self, _: [u8; 2], _: &[u8], _: std::time::Instant) {
            self.0 += 1;
        }
        fn reset(&mut self, _: Option<&str>) {
            self.0 = 0;
        }
    }
    #[test]
    fn disabled_consumers_ignore_deliveries_retained_by_the_pipeline() {
        let consumer = Arc::new(Mutex::new(Counter::default()));
        let mut subscription = WidgetSubscription::new(consumer.clone(), false);
        assert!(!subscription.enabled());
        let retained = subscription.gate.clone();
        // Model a delivery whose client list was copied before deactivation.
        retained.lock().unwrap().enabled = true;
        retained
            .lock()
            .unwrap()
            .message([0x34, 0x36], &[], std::time::Instant::now());
        assert_eq!(consumer.lock().unwrap().0, 1);
        retained.lock().unwrap().enabled = false;
        assert!(!retained.lock().unwrap().accepts([0x34, 0x36], None));
        retained
            .lock()
            .unwrap()
            .message([0x34, 0x36], &[], std::time::Instant::now());
        retained.lock().unwrap().reset(None);
        assert_eq!(consumer.lock().unwrap().0, 1);
        subscription.set_enabled(false);
    }
}
