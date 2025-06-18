//! Helpers for generating an underline.

use void_public::{
    Aspect, ComponentBuilder, Transform, Vec2, bundle_for_builder,
    colors::palette,
    graphics::{TextureId, TextureRender},
    linalg::Vec3,
};

use crate::{Underline, math::ZeroToHundredPercent};

pub const UNDERLINE_OFFSET_Y_PERCENT: ZeroToHundredPercent = ZeroToHundredPercent::new(0.05);
pub const UNDERLINE_HEIGHT_Y_PERCENT: ZeroToHundredPercent = ZeroToHundredPercent::new(0.005);
pub const UNDERLINE_DEFAULT_WIDTH_X_PERCENT: ZeroToHundredPercent = ZeroToHundredPercent::new(0.15);

pub fn create_underline(
    position: Vec3,
    width_percent: Option<ZeroToHundredPercent>,
    aspect: &Aspect,
) -> ComponentBuilder {
    let texture_render = TextureRender {
        texture_id: TextureId(0),
        visible: true,
    };
    let transform = Transform {
        position,
        scale: Vec2::new(
            *width_percent.unwrap_or(UNDERLINE_DEFAULT_WIDTH_X_PERCENT) * aspect.width,
            *UNDERLINE_HEIGHT_Y_PERCENT * aspect.height,
        )
        .into(),
        ..Default::default()
    };
    let color = palette::WHITE;
    bundle_for_builder!(texture_render, transform, color, Underline).into()
}
