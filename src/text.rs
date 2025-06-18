//! Helpers for generating entities with the proper components to create text.

use std::{ffi::CStr, str::from_utf8};

use game_asset::resource_managers::material_manager::materials::MaterialType;
use void_public::{
    Component, ComponentBuilder, Transform, Vec3, bundle_for_builder,
    colors::{Color, palette},
    graphics::TextRender,
    linalg::{Vec2, Vec4},
    text::TextAlignment,
};

use crate::{
    CustomText, HeaderText, RegularText,
    local_error::{LocalError, Result},
};

pub const fn title_from_material_type(material_type: &MaterialType) -> &str {
    match material_type {
        MaterialType::Sprite => "Sprite Material",
        MaterialType::PostProcessing => "Post Processing Material",
    }
}

#[derive(Debug)]
pub enum TextTypes {
    Header,
    Regular,
    Custom(f32),
}

impl TextTypes {
    pub const fn font_size(&self) -> f32 {
        match &self {
            TextTypes::Header => 128.,
            TextTypes::Regular => 64.,
            TextTypes::Custom(font_size) => *font_size,
        }
    }
}

pub fn cstr_to_u8_array<const N: usize>(cstr: &CStr) -> [u8; N] {
    let mut output_array = [0; N];
    cstr.to_bytes_with_nul()
        .iter()
        .take(N)
        .enumerate()
        .for_each(|(index, byte)| output_array[index] = *byte);

    output_array
}

pub fn str_to_u8_array<const N: usize>(str: &str) -> [u8; N] {
    let mut output_array = [0; N];
    str.as_bytes()
        .iter()
        .take(N)
        .enumerate()
        .for_each(|(index, byte)| output_array[index] = *byte);
    output_array
}

pub fn u8_array_to_str(u8_slice: &[u8]) -> Result<&str> {
    from_utf8(u8_slice)
        .map(|str| str.trim_matches('\0'))
        .map_err(|err| err.into())
}

pub fn u8_array_to_cstr(u8_slice: &[u8]) -> Result<&CStr> {
    let nul_position = u8_slice.iter().position(|b| *b == 0).ok_or::<LocalError>(
        "Could not find nul terminator on CStr represented as a u8 array".into(),
    )?;
    let cstr_slice = &u8_slice[..=nul_position];

    unsafe { Ok(CStr::from_bytes_with_nul_unchecked(cstr_slice)) }
}

#[derive(Debug)]
pub struct CreateTextInput<S: AsRef<str>> {
    pub text: S,
    pub visible: bool,
    pub bounds_size: Vec2,
    pub alignment: TextAlignment,
    pub position: Vec3,
    pub color: Vec4,
    pub text_type: TextTypes,
}

impl<S: AsRef<str> + Default> Default for CreateTextInput<S> {
    fn default() -> Self {
        Self {
            text: S::default(),
            visible: true,
            bounds_size: void_public::Vec2::new(0., 0.).into(),
            alignment: TextAlignment::Center,
            position: Vec3::new(0., 0., 0.),
            color: *palette::WHITE,
            text_type: TextTypes::Regular,
        }
    }
}

pub fn create_new_text<S: AsRef<str>, TextType: Component>(
    create_text_input: CreateTextInput<S>,
) -> ComponentBuilder {
    let CreateTextInput {
        text,
        visible,
        bounds_size,
        alignment,
        position,
        color,
        text_type,
    } = create_text_input;
    let text = str_to_u8_array(text.as_ref());
    let text_render = TextRender {
        text,
        visible,
        bounds_size,
        font_size: text_type.font_size(),
        alignment,
    };
    let transform = Transform {
        position: position.into(),
        ..Default::default()
    };
    let color = Color::from(color);
    let mut component_builder: ComponentBuilder =
        bundle_for_builder!(text_render, transform, color).into();
    match text_type {
        TextTypes::Header => component_builder.add_component(HeaderText),
        TextTypes::Regular => component_builder.add_component(RegularText),
        TextTypes::Custom(_) => component_builder.add_component(CustomText),
    }
    component_builder
}

#[cfg(test)]
mod test {
    use crate::text::{str_to_u8_array, u8_array_to_str};

    #[test]
    fn u8_array_isnt_padded_when_converted_back_to_str() {
        let test_str = "hello";
        assert_eq!(test_str, "hello");

        let test_u8_array = str_to_u8_array::<256>(test_str);
        assert_eq!(u8_array_to_str(&test_u8_array).unwrap(), test_str);
    }
}
