mod code;
mod house;
mod settings;
pub(crate) use code::CodeIcon;
pub(crate) use house::HouseIcon;
pub(crate) use settings::SettingsIcon;

mod cross;
mod square;
mod window_minimize;
mod window_restore;

use gpui::{Bounds, Pixels, Point, point, px};

pub(crate) use cross::CrossIcon;
pub(crate) use square::SquareIcon;
pub(crate) use window_minimize::WindowMinimizeIcon;
pub(crate) use window_restore::WindowRestoreIcon;

pub(super) struct ViewBox {
    left: f32,
    top: f32,
    min_x: f32,
    min_y: f32,
    scale_x: f32,
    scale_y: f32,
}

impl ViewBox {
    pub(super) fn new(
        bounds: Bounds<Pixels>,
        min_x: f32,
        min_y: f32,
        width: f32,
        height: f32,
    ) -> Self {
        let scale = (bounds.size.width.as_f32() / width).min(bounds.size.height.as_f32() / height);
        let rendered_width = width * scale;
        let rendered_height = height * scale;

        Self {
            left: bounds.origin.x.as_f32() + (bounds.size.width.as_f32() - rendered_width) / 2.0,
            top: bounds.origin.y.as_f32() + (bounds.size.height.as_f32() - rendered_height) / 2.0,
            min_x,
            min_y,
            scale_x: scale,
            scale_y: scale,
        }
    }

    pub(super) fn point(&self, x: f32, y: f32) -> Point<Pixels> {
        point(
            px(self.left + (x - self.min_x) * self.scale_x),
            px(self.top + (y - self.min_y) * self.scale_y),
        )
    }

    #[allow(dead_code)] // Used by generated icons containing SVG arc commands.
    pub(super) fn radii(&self, x: f32, y: f32) -> Point<Pixels> {
        point(px(x * self.scale_x), px(y * self.scale_y))
    }
}
