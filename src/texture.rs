//! Helpers for generating quads with a texture on them.

use void_public::{
    ComponentBuilder, Transform, Vec2, bundle_for_builder,
    colors::Color,
    graphics::{TextureId, TextureRender},
    linalg::{Vec3, Vec4},
};

const DEFAULT_SCALE: f32 = 100.;

pub fn create_new_texture(
    position: Vec3,
    color: Vec4,
    texture_id: TextureId,
    scale: Option<Vec2>,
) -> ComponentBuilder {
    let texture_render = TextureRender {
        texture_id,
        visible: true,
    };
    let transform = Transform {
        position,
        scale: scale.unwrap_or(Vec2::splat(DEFAULT_SCALE)).into(),
        ..Default::default()
    };
    bundle_for_builder!(texture_render, transform, Color::from(color)).into()
}
