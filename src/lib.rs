//! There are several systems where each showcases some feature or example use of a shader,
//! followed by a menu or input system for interactively selecting between the examples.

use std::{
    env::args,
    error::Error,
    ffi::CStr,
    fmt::{Debug, Display},
    num::NonZero,
    ops::{Add, AddAssign, ControlFlow, Deref},
    path::PathBuf,
};

use array::array_from_iterator;
use asset_registering::register_material;
use game_asset::{
    ecs_module::{GpuInterface, TextAssetManager},
    resource_managers::{
        material_manager::{
            material_parameters_extension::MaterialParametersExt,
            materials::MaterialType,
            uniforms::{MaterialUniforms, UniformValue},
        },
        text_asset_manager::MISSING_TEXT_ID,
    },
    world_render_manager::WorldRenderManager,
};
use game_module_macro::{Component, Resource, set_system_enabled, system, system_once};
use input_handlers::{
    is_back_just_pressed, is_down_just_pressed, is_left_just_pressed, is_right_just_pressed,
    is_select_just_pressed, is_up_just_pressed,
};
use log::{error, warn};
use math::{
    division_result, generate_equal_parts_rotation_matrix, screen_space_coordinate_by_percent,
};
use rand::{Rng, thread_rng};
use serde_big_array::BigArray;
use snapshot::{Deserialize, Serialize};
use text::{
    CreateTextInput, TextTypes, create_new_text, cstr_to_u8_array, str_to_u8_array,
    title_from_material_type, u8_array_to_cstr, u8_array_to_str,
};
use texture::create_new_texture;
use underline::{UNDERLINE_OFFSET_Y_PERCENT, create_underline};
use void_public::{
    Aspect, Component, ComponentId, EcsType, Engine, EntityId, EventReader, EventWriter,
    FrameConstants, Mat2, Query, Resource, Transform, Vec2, Vec3, Vec4, bundle, bundle_for_builder,
    colors::{Color, palette},
    event::{
        TransformT, Vec2T, Vec3T,
        graphics::{
            ColorT, DrawCircle, DrawCircleT, DrawLine, DrawLineT, DrawRectangle,
            DrawRectangleBuilder, DrawText, DrawTextBuilder, MaterialIdFromTextId, NewText,
            NewTexture, TextAlignment,
        },
        input::KeyCode,
    },
    graphics::{TextRender, TextureId, TextureRender},
    input::InputState,
    material::{DefaultMaterials, MaterialId, MaterialParameters},
    text::TextId,
};

pub mod array;
pub mod asset_registering;
pub mod input_handlers;
pub mod local_error;
pub mod math;
#[cfg(test)]
pub(crate) mod test_validation;
pub mod text;
pub mod texture;
pub mod underline;

#[system_once]
fn turn_off_systems() {
    set_system_enabled!(false, handle_assets_loaded);
}

#[system_once]
// We probably need some helper code to have systems start off if desired
fn turn_off_material_test_systems() {
    set_system_enabled!(
        false,
        invert_y_startup_system,
        invert_y_system,
        test_post_startup_system,
        test_post_system,
        warp_startup_system,
        warp_system,
        channel_inspector_startup_system,
        color_replacement_startup_system,
        color_replacement_system,
        desat_sprite_startup_system,
        pan_sprite_startup_system,
        scrolling_color_startup_system,
        scrolling_color_system,
        starfield_startup_system,
        starfield_system,
        immediate_mode_test,
        stress_test_startup_system,
        stress_test_system,
    );
}

#[system_once]
/// This system sets up all material tests. [`MaterialTest`]'s should all be created in this system,
/// along with any supporting [`Material`]'s and textures that the [`MaterialTest`] may need.
///
/// Please note, this system currently accesses [`GpuResource`] and [`PipelineManager`] from `gpu_web`, which is not the proper
/// way that a module should access the engine. `gpu_web` is a platform implementation for [`GpuResource`]. In the future,
/// [`PipelineManager`] will be moved to `void_public` and [`AssetManager`] will be expanded to properly load textures.
fn materials_setup(
    gpu_interface: &mut GpuInterface,
    material_test_id_holder: &mut MaterialTestIdHolder,
    text_asset_manager: &mut TextAssetManager,
    new_texture_event_writer: EventWriter<NewTexture>,
    new_text_event_writer: EventWriter<NewText<'_>>,
    view: &mut View,
) {
    let pending_texture = gpu_interface
        .texture_asset_manager
        .load_texture(
            &PathBuf::from("textures/arrow_up.png").into(),
            true,
            &new_texture_event_writer,
        )
        .unwrap();
    Engine::spawn(bundle!(&MaterialTextureAsset::new(pending_texture.id())));

    let pending_texture = gpu_interface
        .texture_asset_manager
        .load_texture(
            &PathBuf::from("textures/random.png").into(),
            false,
            &new_texture_event_writer,
        )
        .unwrap();
    Engine::spawn(bundle!(&MaterialTextureAsset::new(pending_texture.id())));

    let pending_texture = gpu_interface
        .texture_asset_manager
        .load_texture(
            &PathBuf::from("textures/scared.png").into(),
            true,
            &new_texture_event_writer,
        )
        .unwrap();
    Engine::spawn(bundle!(&MaterialTextureAsset::new(pending_texture.id())));

    let pending_texture = gpu_interface
        .texture_asset_manager
        .load_texture(
            &PathBuf::from("textures/star_map_with_mask.png").into(),
            false,
            &new_texture_event_writer,
        )
        .unwrap();
    Engine::spawn(bundle!(&MaterialTextureAsset::new(pending_texture.id())));

    let (_, invert_y_y_test_id) = register_material(
        "invert_y",
        MaterialType::PostProcessing,
        &"toml_materials/post_processing/invert_y.toml".into(),
        c"invert_y_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );
    let (_, test_post_test_id) = register_material(
        "test_post",
        MaterialType::PostProcessing,
        &"toml_materials/post_processing/test_post.toml".into(),
        c"test_post_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );
    let (_, warp_test_id) = register_material(
        "warp",
        MaterialType::PostProcessing,
        &"toml_materials/post_processing/warp.toml".into(),
        c"warp_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );

    let (_, channel_inspector_test_id) = register_material(
        "channel_inspector",
        MaterialType::Sprite,
        &"toml_materials/sprite/channel_inspector.toml".into(),
        c"channel_inspector_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );
    let (_, color_replacement_test_id) = register_material(
        "color_replacement",
        MaterialType::Sprite,
        &"toml_materials/sprite/color_replacement.toml".into(),
        c"color_replacement_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );
    let (desat_sprite_text_id, desat_sprite_test_id) = register_material(
        "desat_sprite",
        MaterialType::Sprite,
        &"toml_materials/sprite/desat_sprite.toml".into(),
        c"desat_sprite_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );
    let (pan_sprite_text_id, pan_sprite_test_id) = register_material(
        "pan_sprite",
        MaterialType::Sprite,
        &"toml_materials/sprite/pan_sprite.toml".into(),
        c"pan_sprite_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );
    let (_, scrolling_color_test_id) = register_material(
        "scrolling_color",
        MaterialType::Sprite,
        &"toml_materials/sprite/scrolling_color.toml".into(),
        c"scrolling_color_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );
    let (_, starfield_test_id) = register_material(
        "starfield",
        MaterialType::Sprite,
        &"toml_materials/sprite/starfield.toml".into(),
        c"starfield_startup_system",
        gpu_interface,
        material_test_id_holder,
        &new_text_event_writer,
        text_asset_manager,
    );

    let material_ids = &[
        MaybeLoadedMaterial::new(MaterialType::Sprite, desat_sprite_text_id),
        MaybeLoadedMaterial::new(MaterialType::Sprite, pan_sprite_text_id),
        MaybeLoadedMaterial::new_material_loaded(
            MaterialType::Sprite,
            DefaultMaterials::Sprite.material_id(),
        ),
    ];

    let stress_test_material_test = &MaterialTest::new(
        "stress_test",
        c"stress_test_startup_system",
        material_ids,
        &MaterialType::Sprite,
        material_test_id_holder,
    );
    Engine::spawn(bundle!(stress_test_material_test));

    let immediate_mode_test_material_test = &MaterialTest::new(
        "immediate_mode_test",
        c"immediate_mode_test",
        material_ids,
        &MaterialType::Sprite,
        material_test_id_holder,
    );
    Engine::spawn(bundle!(immediate_mode_test_material_test));

    let args = args().collect::<Vec<String>>();
    if args.len() > 1 {
        let test_name = &args[1];
        let test_id = match test_name.to_lowercase().as_str() {
            "invert_y" => Some((MaterialType::PostProcessing, invert_y_y_test_id)),
            "test_post" => Some((MaterialType::PostProcessing, test_post_test_id)),
            "warp" => Some((MaterialType::PostProcessing, warp_test_id)),
            "channel_inspector" => Some((MaterialType::Sprite, channel_inspector_test_id)),
            "color_replacement" => Some((MaterialType::Sprite, color_replacement_test_id)),
            "desat_sprite" => Some((MaterialType::Sprite, desat_sprite_test_id)),
            "pan_sprite" => Some((MaterialType::Sprite, pan_sprite_test_id)),
            "scrolling_color" => Some((MaterialType::Sprite, scrolling_color_test_id)),
            "starfield" => Some((MaterialType::Sprite, starfield_test_id)),
            "immediate_mode_test" => {
                Some((MaterialType::Sprite, immediate_mode_test_material_test.id()))
            }
            "stress_test" => Some((MaterialType::Sprite, stress_test_material_test.id())),
            _ => None,
        };
        if let Some((material_type, test_id)) = test_id {
            view.post_load_transition = Some(TransitionTo::Material((material_type, test_id)));
        }
    }

    view.set_transition_to(TransitionTo::Loading);
    set_system_enabled!(true, handle_assets_loaded);
}

#[system]
fn handle_material_id_from_text_id_events(
    mut material_test_assets: Query<&mut MaterialTest>,
    material_id_from_text_id_events: EventReader<MaterialIdFromTextId>,
) {
    for material_id_from_text_id_event in &material_id_from_text_id_events {
        material_test_assets.for_each(|material_test_asset| {
            let text_id =
                TextId(unsafe { NonZero::new_unchecked(material_id_from_text_id_event.text_id()) });
            let material_id = MaterialId(material_id_from_text_id_event.material_id());
            material_test_asset.update_maybe_loaded_materials(text_id, material_id);
            Engine::spawn(bundle!(&MaterialAsset::new(material_id)));
        });
    }
}

#[system]
fn handle_assets_loaded(
    gpu_interface: &GpuInterface,
    text_asset_manager: &TextAssetManager,
    mut material_assets: Query<(&EntityId, &MaterialAsset)>,
    mut material_text_assets: Query<(&EntityId, &MaterialTextAsset)>,
    mut material_texture_assets: Query<(&EntityId, &MaterialTextureAsset)>,
    view: &mut View,
) {
    let texture_ids_iter = material_texture_assets.iter().map(|query_components_ref| {
        let (_, material_texture_asset) = query_components_ref.unpack();
        material_texture_asset.texture_id()
    });
    let text_ids_iter = material_text_assets.iter().map(|query_components_ref| {
        let (_, material_text_asset) = query_components_ref.unpack();
        material_text_asset.text_id()
    });
    let pipeline_ids =
        material_assets
            .iter()
            .fold(vec![], |mut accumulator, query_components_ref| {
                let (_, material_asset) = query_components_ref.unpack();
                let material_id = material_asset.material_id();
                if let Some(pipeline_id) = gpu_interface
                    .pipeline_asset_manager
                    .get_pipeline_id_from_material_id(*material_id)
                {
                    accumulator.push(pipeline_id);
                }
                accumulator
            });
    if gpu_interface
        .texture_asset_manager
        .are_all_ids_loaded(texture_ids_iter)
        && text_asset_manager.are_all_ids_loaded(text_ids_iter)
        && !pipeline_ids.is_empty()
        && gpu_interface
            .pipeline_asset_manager
            .are_all_ids_loaded(pipeline_ids.iter())
    {
        view.set_transition_to(match view.post_load_transition {
            Some(transition_to) => transition_to,
            None => TransitionTo::MainView,
        });

        view.post_load_transition = None;

        material_texture_assets.for_each(|(entity_id, _)| {
            Engine::despawn(**entity_id);
        });

        material_text_assets.for_each(|(entity_id, _)| {
            Engine::despawn(**entity_id);
        });

        material_assets.for_each(|(entity_id, _)| {
            Engine::despawn(**entity_id);
        });

        set_system_enabled!(
            false,
            handle_assets_loaded,
            handle_material_id_from_text_id_events
        );
    }
}

#[system_once]
fn channel_inspector_startup_system(
    aspect: &Aspect,
    material_test_query: Query<&MaterialTest>,
    gpu_interface: &GpuInterface,
) {
    let Some(channel_inspector_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "channel_inspector")
    else {
        error!("Could not find channel_inspector material test");
        return;
    };
    let Some(Some(material_id)) = channel_inspector_material_test.material_id_iter().next() else {
        error!("Could not find material id on channel inspector");
        return;
    };

    let star_map_texture_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/star_map_with_mask.png".into())
        .unwrap()
        .id();

    let channel_images_scale = Vec2::splat(aspect.width * 0.1);

    let mut base_material_params = MaterialParameters::new(material_id)
        .update_texture(
            &gpu_interface.material_manager,
            &("map", &star_map_texture_id),
        )
        .unwrap()
        .end_chain();

    let channel_names = ["red", "green", "blue", "alpha"];

    for (index, channel_name) in channel_names.into_iter().enumerate() {
        let channel_value = index as f32;
        let channel_material_params = base_material_params
            .update_uniform(
                &gpu_interface.material_manager,
                &("channel", &channel_value.into()),
            )
            .unwrap()
            .end_chain();

        let x_percent = 0.125 + 2. * 0.125 * channel_value;
        let texture_position =
            screen_space_coordinate_by_percent(aspect, x_percent.into(), 0.5.into()).extend(0.);
        let mut texture_component_builder = create_new_texture(
            texture_position.into(),
            *palette::WHITE,
            star_map_texture_id,
            Some(channel_images_scale),
        );
        texture_component_builder.add_components(bundle_for_builder!(
            MaterialTestObject,
            channel_material_params
        ));
        Engine::spawn(&texture_component_builder.build());

        let mut text_component_builder = create_new_text::<_, RegularText>(CreateTextInput {
            position: texture_position - Vec3::new(0., aspect.height * 0.2, 0.),
            text: channel_name,
            ..Default::default()
        });
        text_component_builder.add_component(MaterialTestObject);
        Engine::spawn(&text_component_builder.build());
    }
}

#[system_once]
fn color_replacement_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    material_test_query: Query<&mut MaterialTest>,
) {
    let Some(channel_inspector_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "color_replacement")
    else {
        error!("Could not find color_replacement material test");
        return;
    };
    let Some(Some(material_id)) = channel_inspector_material_test.material_id_iter().next() else {
        error!("Could not find material id on color_replacement");
        return;
    };

    let white_color_uniform: UniformValue = (*palette::WHITE).get().into();
    let grey_color_uniform = (*palette::GRAY).get().into();
    let scared_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/scared.png".into())
        .unwrap()
        .id();

    let material_params = MaterialParameters::new(material_id)
        .update_uniforms(
            &gpu_interface.material_manager,
            &[
                ("color_to_replace", &white_color_uniform),
                ("color_to_insert", &grey_color_uniform),
            ],
        )
        .unwrap()
        .update_texture(&gpu_interface.material_manager, &("color_tex", &scared_id))
        .unwrap()
        .end_chain();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        scared_id,
        Some(Vec2::splat(aspect.width * 0.25)),
    );
    texture_component_builder.add_components(bundle_for_builder!(
        MaterialTestObject,
        material_params,
        TimePassedSinceCreation::default()
    ));
    Engine::spawn(&texture_component_builder.build());

    let mut text_component_builder = create_new_text::<_, HeaderText>(CreateTextInput {
        position: screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.75.into()).extend(0.),
        text: "Test",
        ..Default::default()
    });
    text_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&text_component_builder.build());
    set_system_enabled!(true, color_replacement_system);
}

#[system]
fn color_replacement_system(
    frame_constants: &FrameConstants,
    gpu_interface: &GpuInterface,
    mut textures: Query<(
        &TextureRender,
        &mut TimePassedSinceCreation,
        &mut MaterialParameters,
    )>,
) {
    textures.for_each(|(_, time_passed_since_creation, material_params)| {
        *time_passed_since_creation += frame_constants.delta_time;

        let new_target_color: UniformValue = Vec4::new(
            f32::sin(***time_passed_since_creation * 0.1).abs(),
            f32::cos(***time_passed_since_creation * 0.5).abs(),
            f32::sin(***time_passed_since_creation * 0.3 + 0.5).abs(),
            1.,
        )
        .into();

        material_params
            .update_uniform(
                &gpu_interface.material_manager,
                &("color_to_insert", &new_target_color),
            )
            .unwrap();
    });
}

#[system_once]
fn pan_sprite_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    material_test_query: Query<&MaterialTest>,
) {
    let Some(pan_sprite_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "pan_sprite")
    else {
        error!("Could not find pan_sprite material test");
        return;
    };
    let Some(Some(material_id)) = pan_sprite_material_test.material_id_iter().next() else {
        error!("Could not find material id on pan_sprite");
        return;
    };

    let arrow_up_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/arrow_up.png".into())
        .unwrap()
        .id();

    let material_params = MaterialParameters::new(material_id)
        .update_texture(
            &gpu_interface.material_manager,
            &("color_tex", &arrow_up_id),
        )
        .unwrap()
        .end_chain();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        arrow_up_id,
        Some(Vec2::splat(aspect.width * 0.15)),
    );
    texture_component_builder
        .add_components(bundle_for_builder!(MaterialTestObject, material_params));
    Engine::spawn(&texture_component_builder.build());

    let mut text_component_builder = create_new_text::<_, HeaderText>(CreateTextInput {
        position: screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.75.into()).extend(0.),
        text: "Test",
        ..Default::default()
    });
    text_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&text_component_builder.build());
}

#[system_once]
fn desat_sprite_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    material_test_query: Query<&MaterialTest>,
) {
    let Some(desat_sprite_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "desat_sprite")
    else {
        error!("Could not find desat_sprite material test");
        return;
    };
    let Some(Some(material_id)) = desat_sprite_material_test.material_id_iter().next() else {
        error!("Could not find material id on desat_sprite");
        return;
    };

    let arrow_up_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/arrow_up.png".into())
        .unwrap()
        .id();

    let material_params = MaterialParameters::new(material_id)
        .update_texture(
            &gpu_interface.material_manager,
            &("color_tex", &arrow_up_id),
        )
        .unwrap()
        .end_chain();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        arrow_up_id,
        Some(Vec2::splat(aspect.width * 0.15)),
    );

    texture_component_builder
        .add_components(bundle_for_builder!(MaterialTestObject, material_params));
    Engine::spawn(&texture_component_builder.build());

    let mut text_component_builder = create_new_text::<_, HeaderText>(CreateTextInput {
        position: screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.75.into()).extend(0.),
        text: "Test",
        ..Default::default()
    });
    text_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&text_component_builder.build());
}

const SCROLLING_COLOR_SCROLL_SPEED_CENTER_POINT: f32 = 1.;

#[system_once]
fn scrolling_color_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    material_test_query: Query<&MaterialTest>,
) {
    let Some(scrolling_color_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "scrolling_color")
    else {
        error!("Could not find scrolling_color material test");
        return;
    };
    let Some(Some(material_id)) = scrolling_color_material_test.material_id_iter().next() else {
        error!("Could not find material id on scrolling_color");
        return;
    };

    let material_params = MaterialParameters::new(material_id)
        .update_uniforms(
            &gpu_interface.material_manager,
            &[
                ("time", &0.0.into()),
                (
                    "scroll_speed",
                    &SCROLLING_COLOR_SCROLL_SPEED_CENTER_POINT.into(),
                ),
            ],
        )
        .unwrap()
        .end_chain();

    let scared_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/scared.png".into())
        .unwrap()
        .id();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        scared_id,
        Some(Vec2::splat(aspect.width * 0.15)),
    );
    texture_component_builder.add_components(bundle_for_builder!(
        MaterialTestObject,
        material_params,
        TimePassedSinceCreation::default()
    ));
    Engine::spawn(&texture_component_builder.build());

    let mut text_component_builder = create_new_text::<_, HeaderText>(CreateTextInput {
        position: screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.75.into()).extend(0.),
        text: "Test",
        ..Default::default()
    });
    text_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&text_component_builder.build());
    set_system_enabled!(true, scrolling_color_system);
}

#[system]
fn scrolling_color_system(
    frame_constants: &FrameConstants,
    gpu_interface: &GpuInterface,
    mut textures: Query<(
        &TextureRender,
        &mut TimePassedSinceCreation,
        &mut MaterialParameters,
    )>,
) {
    textures.for_each(|(_, time_passed_since_creation, material_params)| {
        *time_passed_since_creation += frame_constants.delta_time;

        let current_speed = SCROLLING_COLOR_SCROLL_SPEED_CENTER_POINT
            + 0.75 * f32::sin(***time_passed_since_creation * 0.001);

        material_params
            .update_uniforms(
                &gpu_interface.material_manager,
                &[
                    ("time", &(***time_passed_since_creation).into()),
                    ("scroll_speed", &current_speed.into()),
                ],
            )
            .unwrap();
    });
}

#[system_once]
fn starfield_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    material_test_query: Query<&MaterialTest>,
) {
    let Some(starfield_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "starfield")
    else {
        error!("Could not find starfield material test");
        return;
    };
    let Some(Some(material_id)) = starfield_material_test.material_id_iter().next() else {
        error!("Could not find material id on starfield");
        return;
    };
    let material = gpu_interface
        .material_manager
        .get_material(material_id)
        .unwrap();

    let random_texture = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/random.png".into())
        .unwrap();
    let random_texture = random_texture.as_loaded_texture().unwrap();
    let star_map_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/star_map_with_mask.png".into())
        .unwrap()
        .id();

    let material_params = material
        .generate_default_material_parameters()
        .update_uniform(
            &gpu_interface.material_manager,
            &(
                "texture_height",
                &(random_texture.height() as f32 * aspect.height * 0.1).into(),
            ),
        )
        .unwrap()
        .update_textures(
            &gpu_interface.material_manager,
            &[("star_map", &star_map_id), ("random", &random_texture.id())],
        )
        .unwrap()
        .end_chain();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        star_map_id,
        Some(Vec2::splat(aspect.width * 0.325)),
    );
    texture_component_builder.add_components(bundle_for_builder!(
        MaterialTestObject,
        material_params,
        TimePassedSinceCreation::default()
    ));
    Engine::spawn(&texture_component_builder.build());
    set_system_enabled!(true, starfield_system);
}

#[system]
fn starfield_system(
    frame_constants: &FrameConstants,
    gpu_interface: &GpuInterface,
    input_state: &InputState,
    material_test_query: Query<&MaterialTest>,
    mut textures: Query<(
        &TextureRender,
        &mut TimePassedSinceCreation,
        &mut MaterialParameters,
    )>,
) {
    let Some(starfield_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "starfield")
    else {
        error!("Could not find starfield material test");
        return;
    };
    let Some(Some(material_id)) = starfield_material_test.material_id_iter().next() else {
        error!("Could not find material id on starfield");
        return;
    };
    let material = gpu_interface
        .material_manager
        .get_material(material_id)
        .unwrap();
    textures.for_each(|(_, time_passed_since_creation, material_params)| {
        *time_passed_since_creation += frame_constants.delta_time;
        let current_uniforms = material
            .get_current_uniforms(&material_params.data)
            .unwrap();
        let current_speed = current_uniforms.get("speed").unwrap();
        let current_speed = match current_speed {
            UniformValue::F32(value) => value.current_value(),
            _ => unreachable!(),
        };

        let current_stars = current_uniforms.get("star_number").unwrap();
        let current_stars = match current_stars {
            UniformValue::F32(value) => value.current_value(),
            _ => unreachable!(),
        };

        const SPEED_INCREMENT: f32 = 0.1;

        let new_speed = if is_left_just_pressed(input_state) {
            Some(current_speed - SPEED_INCREMENT)
        } else if is_right_just_pressed(input_state) {
            Some(current_speed + SPEED_INCREMENT)
        } else {
            None
        };

        let speed_burst_value = if input_state.keys[KeyCode::Space].just_pressed() {
            Some(80.0.into())
        } else if input_state.keys[KeyCode::Space].just_released() {
            let default_uniforms = material.generate_default_material_uniforms().unwrap();
            Some(default_uniforms.get("speed").unwrap().clone())
        } else {
            None
        };

        const STARS_INCREMENT: f32 = 5.;
        let new_stars = if is_up_just_pressed(input_state) {
            Some(current_stars + STARS_INCREMENT)
        } else if is_down_just_pressed(input_state) {
            Some(current_stars - STARS_INCREMENT)
        } else {
            None
        };

        let mut material_uniforms = material_params
            .as_material_uniforms(&gpu_interface.material_manager)
            .unwrap();

        if let Some(new_stars) = new_stars {
            material_uniforms
                .update("star_number", new_stars.into())
                .unwrap();
        }

        if let Some(speed_burst_value) = speed_burst_value {
            material_uniforms
                .update("speed", speed_burst_value)
                .unwrap();
        } else if let Some(new_speed) = new_speed {
            material_uniforms.update("speed", new_speed.into()).unwrap();
        }

        material_uniforms
            .update("time_elapsed", (***time_passed_since_creation).into())
            .unwrap();
        material_params
            .update_from_material_uniforms(&material_uniforms)
            .unwrap();
    });
}

#[derive(Debug, Component, serde::Deserialize, serde::Serialize)]
pub struct Velocity {
    pub direction: Vec3,
    pub rotation: f32,
}

#[system]
#[allow(clippy::too_many_arguments)]
fn immediate_mode_test(
    draw_circle_writer: EventWriter<DrawCircle>,
    draw_line_writer: EventWriter<DrawLine>,
    draw_text_writer: EventWriter<DrawText>,
    draw_rectangle_writer: EventWriter<DrawRectangle>,
    aspect: &Aspect,
    frame_constants: &FrameConstants,
    gpu_interface: &GpuInterface,
    mut time_passed_since_creation: Query<&mut TimePassedSinceCreation>,
) {
    let scared_id = match gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/scared.png".into())
    {
        Some(texture) => texture.id(),
        None => {
            warn!(
                "Could not find texture scared.png, if this occurs at the beginning of the first frame it is normal (for now), otherwise this is an error"
            );
            return;
        }
    };

    let scared_distance = Vec2::new(aspect.width * 0.15, 0.);
    let circle_distance = Vec2::new(aspect.width * 0.275, 0.);
    let line_distance = Vec2::new(aspect.width * 0.375, 0.);
    let center_point_vec2 = screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into());
    let center_point_vec3 = center_point_vec2.extend(1.);
    let center_point_vec3t = Vec3T {
        x: center_point_vec3.x,
        y: center_point_vec3.y,
        z: center_point_vec3.z,
    };

    let time_passed = if time_passed_since_creation.is_empty() {
        Engine::spawn(bundle!(
            &MaterialTestObject,
            &TimePassedSinceCreation::default()
        ));
        0.
    } else {
        let mut time_passed = 0.;
        time_passed_since_creation.for_each(|time_passed_since_creation| {
            *time_passed_since_creation += frame_constants.delta_time;
            time_passed = ***time_passed_since_creation;
        });
        time_passed
    };

    draw_text_writer.write_builder(|builder| {
        let flatbuffer_test_string = builder.create_string("This is a test");
        let mut draw_text_builder = DrawTextBuilder::new(builder);
        draw_text_builder.add_font_size(48.);
        draw_text_builder.add_text(flatbuffer_test_string);
        let red = 0.25 * time_passed.sin() + 0.75;
        let green = 0.25 * time_passed.cos() + 0.75;
        draw_text_builder.add_color(&void_public::event::graphics::Color::new(
            red, green, 1., 1.,
        ));
        draw_text_builder.add_bounds(&Vec2T { x: 500., y: 500. }.pack());
        draw_text_builder.add_text_alignment(TextAlignment::Center);
        let transform = TransformT {
            position: center_point_vec3t,
            scale: Vec2T { x: 1., y: 1. },
            ..Default::default()
        };
        draw_text_builder.add_transform(&transform.pack());
        draw_text_builder.add_z(1.);
        draw_text_builder.finish()
    });

    let starting_rotation_matrix = Mat2::from_angle(time_passed);
    let mut rotation_matrix = starting_rotation_matrix;
    let num_of_images = 5;
    let image_shift_rotation_matrix = generate_equal_parts_rotation_matrix(num_of_images as f32);
    for index in 0..num_of_images {
        draw_rectangle_writer.write_builder(|builder| {
            let mut draw_rectangle_builder = DrawRectangleBuilder::new(builder);
            draw_rectangle_builder.add_asset_id(*scared_id);
            let red = 0.25 * (index as f32).cos() + 0.75;
            let green = 0.25 * (index as f32).sin() + 0.75;
            draw_rectangle_builder.add_color(&void_public::event::graphics::Color::new(
                red, green, 1., 1.,
            ));
            let position = center_point_vec3 + (rotation_matrix * scared_distance).extend(0.);
            rotation_matrix *= image_shift_rotation_matrix;
            let transform = TransformT {
                position: Vec3T {
                    x: position.x,
                    y: position.y,
                    z: position.z,
                },
                scale: Vec2T { x: 125., y: 125. },
                rotation: (index as f32 + time_passed).sin(),
                ..Default::default()
            };
            draw_rectangle_builder.add_transform(&transform.pack());
            draw_rectangle_builder.finish()
        });
    }

    rotation_matrix = starting_rotation_matrix;
    let num_of_circles = 6;
    let circle_shift_rotation_matrix = generate_equal_parts_rotation_matrix(num_of_circles as f32);
    for index in 0..num_of_circles {
        let position = center_point_vec2 + (rotation_matrix * circle_distance);
        rotation_matrix *= circle_shift_rotation_matrix;
        let r = 0.25 * (index as f32).sin() + 0.75;
        let g = 0.25 * (index as f32).cos() + 0.75;
        draw_circle_writer.write(
            DrawCircleT {
                position: Vec2T {
                    x: position.x,
                    y: position.y,
                },
                z: 0.,
                radius: 100.,
                subdivisions: 32,
                rotation: 0.,
                color: ColorT { r, g, b: 1., a: 1. },
            }
            .pack(),
        );
    }

    rotation_matrix = starting_rotation_matrix;
    let num_of_lines = 4;
    let half_line_length = 35.;
    let thickness = 20.;
    let line_shift_rotation_matrix = generate_equal_parts_rotation_matrix(num_of_lines as f32);
    for index in 0..num_of_lines {
        let center_position = center_point_vec2 + (rotation_matrix * line_distance);
        rotation_matrix *= line_shift_rotation_matrix;
        let from_position = center_position - Vec2::new(half_line_length, 0.);
        let to_position = center_position + Vec2::new(half_line_length, 0.);
        let r = 0.25 * (index as f32).cos() + 0.75;
        let g = 0.25 * (index as f32).sin() + 0.75;
        draw_line_writer.write(
            DrawLineT {
                from: Vec2T {
                    x: from_position.x,
                    y: from_position.y,
                },
                to: Vec2T {
                    x: to_position.x,
                    y: to_position.y,
                },
                z: 0.,
                thickness,
                color: ColorT { r, g, b: 1., a: 1. },
            }
            .pack(),
        );
    }
}

/// Currently this system uses non deterministic RNG code, once we have a RNG library in the Engine
/// that portion should be replaced
#[system_once]
fn stress_test_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    material_test_query: Query<&MaterialTest>,
) {
    let Some(stress_test_material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "stress_test")
    else {
        error!("Could not find stress_test material test");
        return;
    };
    let mut materials_id_iter = stress_test_material_test.material_id_iter();
    let Some(Some(desat_material_id)) = materials_id_iter.next() else {
        error!("Could not find desat_material_id on stress_test");
        return;
    };
    let Some(Some(pan_material_id)) = materials_id_iter.next() else {
        error!("Could not find pan_material_id on stress_test");
        return;
    };
    let Some(Some(default_sprite_material_id)) = materials_id_iter.next() else {
        error!("Could not find default_sprite_material_id on stress_test");
        return;
    };
    let mut rng = thread_rng();

    let sprite_materials = [
        gpu_interface
            .material_manager
            .get_material(default_sprite_material_id)
            .unwrap(),
        gpu_interface
            .material_manager
            .get_material(pan_material_id)
            .unwrap(),
        gpu_interface
            .material_manager
            .get_material(desat_material_id)
            .unwrap(),
    ];

    let scared_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/scared.png".into())
        .unwrap()
        .id();

    for i in 0..32 {
        let material = sprite_materials[i % sprite_materials.len()];

        let material_params = MaterialParameters::new(material.material_id())
            .update_texture(&gpu_interface.material_manager, &("color_tex", &scared_id))
            .unwrap()
            .end_chain();

        // This scales the velocity with the size of the window, using the
        // width as a shorthand for that
        let velocity_scalar = aspect.width * 0.15;
        let velocity = Velocity {
            direction: Vec3::new(
                rng.gen_range(-velocity_scalar..velocity_scalar),
                rng.gen_range(-velocity_scalar..velocity_scalar),
                0.,
            ),
            rotation: rng.gen_range(-6.0..6.),
        };

        let mut texture_component_builder = create_new_texture(
            Vec3::new(
                rng.gen_range(-1.0..1.) * aspect.width * 0.5,
                rng.gen_range(-1.0..1.) * aspect.height * 0.5,
                1.,
            )
            .into(),
            Vec4::new(
                rng.gen_range(0.5..3.0),
                rng.gen_range(0.5..3.0),
                rng.gen_range(0.5..3.0),
                1.,
            )
            .into(),
            scared_id,
            Some(Vec2::new(
                rng.gen_range(0.25..1.0) * aspect.width * 0.125,
                rng.gen_range(0.25..1.0) * aspect.width * 0.125,
            )),
        );
        texture_component_builder.add_components(bundle_for_builder!(
            MaterialTestObject,
            material_params,
            velocity
        ));
        Engine::spawn(&texture_component_builder.build());
    }
    set_system_enabled!(true, stress_test_system);
}

#[system]
fn stress_test_system(
    aspect: &Aspect,
    frame_constants: &FrameConstants,
    mut test_objects_query: Query<(
        &MaterialTestObject,
        &mut Transform,
        &mut Velocity,
        &mut MaterialParameters,
    )>,
) {
    test_objects_query.for_each(|(_, transform, velocity, _)| {
        transform
            .position
            .set(transform.position.get() + velocity.direction * frame_constants.delta_time);

        let transform_position = transform.position.get();
        if transform_position.x < -aspect.width * 0.5 && velocity.direction.x < 0.
            || transform_position.x > aspect.width * 0.5 && velocity.direction.y > 0.
        {
            velocity.direction.x = -velocity.direction.x;
        }

        if transform_position.y < -aspect.height * 0.5 && velocity.direction.y < 0.
            || transform_position.y > aspect.height * 0.5 && velocity.direction.y > 0.
        {
            velocity.direction.y = -velocity.direction.y;
        }

        transform.rotation += velocity.rotation * frame_constants.delta_time;
    });
}

fn invert_y_scared_distance(aspect: &Aspect) -> Vec2 {
    Vec2::new(aspect.width * 0.3, 0.)
}

#[system_once]
fn invert_y_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    world_render_manager: &mut WorldRenderManager,
    material_test_query: Query<&mut MaterialTest>,
) {
    let scared_distance = invert_y_scared_distance(aspect);
    let Some(material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "invert_y")
    else {
        error!("Could not find invert_y material test");
        return;
    };
    let Some(Some(material_id)) = material_test.material_id_iter().next() else {
        error!("invert_y material test is missing expected material_id");
        return;
    };

    let material = gpu_interface
        .material_manager
        .get_material(material_id)
        .unwrap();
    let material_uniforms = MaterialUniforms::empty(material_id);

    world_render_manager.add_or_update_postprocess(material, &material_uniforms);

    let arrow_up_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/arrow_up.png".into())
        .unwrap()
        .id();
    let scared_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/scared.png".into())
        .unwrap()
        .id();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        arrow_up_id,
        Some(Vec2::splat(aspect.width * 0.08)),
    );
    texture_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&texture_component_builder.build());

    let mut texture_component_builder = create_new_texture(
        scared_distance.extend(0.).into(),
        *palette::WHITE,
        scared_id,
        Some(Vec2::splat(aspect.width * 0.11)),
    );
    texture_component_builder.add_components(bundle_for_builder!(
        MaterialTestObject,
        TimePassedSinceCreation::default()
    ));
    Engine::spawn(&texture_component_builder.build());

    let mut text_component_builder = create_new_text::<_, HeaderText>(CreateTextInput {
        position: screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.7.into()).extend(0.),
        text: "This is up",
        ..Default::default()
    });
    text_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&text_component_builder.build());
    set_system_enabled!(true, invert_y_system);
}

#[system]
fn invert_y_system(
    aspect: &Aspect,
    frame_constants: &FrameConstants,
    mut texture_query: Query<(&mut Transform, &TextureRender, &mut TimePassedSinceCreation)>,
) {
    let scared_distance = invert_y_scared_distance(aspect);
    texture_query.for_each(|(transform, _, time_passed_since_creation)| {
        *time_passed_since_creation += frame_constants.delta_time;
        let rotation_matrix = Mat2::from_angle(***time_passed_since_creation);
        transform.position = (rotation_matrix * scared_distance).extend(0.).into();
        transform.rotation += (***time_passed_since_creation).cos() / 8.;
    });
}

fn test_post_scared_distance(aspect: &Aspect) -> Vec2 {
    Vec2::new(aspect.width * 0.3, 0.)
}

#[system_once]
fn test_post_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    world_render_manager: &mut WorldRenderManager,
    material_test_query: Query<&MaterialTest>,
) {
    let scared_distance = test_post_scared_distance(aspect);
    let Some(material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "test_post")
    else {
        error!("Could not find test_post material test");
        return;
    };
    let Some(Some(material_id)) = material_test.material_id_iter().next() else {
        error!("test_post material test is missing expected material_id");
        return;
    };

    let material = gpu_interface
        .material_manager
        .get_material(material_id)
        .unwrap();

    let material_uniforms = MaterialUniforms::empty(material_id);

    world_render_manager.add_or_update_postprocess(material, &material_uniforms);

    let arrow_up_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/arrow_up.png".into())
        .unwrap()
        .id();
    let scared_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/scared.png".into())
        .unwrap()
        .id();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        arrow_up_id,
        Some(Vec2::splat(aspect.width * 0.08)),
    );
    texture_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&texture_component_builder.build());

    let mut texture_component_builder = create_new_texture(
        scared_distance.extend(0.).into(),
        *palette::WHITE,
        scared_id,
        Some(Vec2::splat(aspect.width * 0.11)),
    );

    texture_component_builder.add_components(bundle_for_builder!(
        MaterialTestObject,
        TimePassedSinceCreation::default()
    ));
    Engine::spawn(&texture_component_builder.build());

    let mut text_component_builder = create_new_text::<_, HeaderText>(CreateTextInput {
        position: screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.7.into()).extend(0.),
        text: "This is up",
        ..Default::default()
    });
    text_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&text_component_builder.build());
    set_system_enabled!(true, test_post_system);
}

#[system]
fn test_post_system(
    aspect: &Aspect,
    frame_constants: &FrameConstants,
    mut texture_query: Query<(&mut Transform, &TextureRender, &mut TimePassedSinceCreation)>,
) {
    let scared_distance = test_post_scared_distance(aspect);
    texture_query.for_each(|(transform, _, time_passed_since_creation)| {
        *time_passed_since_creation += frame_constants.delta_time;
        let rotation_matrix = Mat2::from_angle(***time_passed_since_creation);
        transform.position = (rotation_matrix * scared_distance).extend(0.).into();
        transform.rotation += (***time_passed_since_creation).cos() / 8.;
    });
}

fn warp_scared_distance(aspect: &Aspect) -> Vec2 {
    Vec2::new(aspect.width * 0.3, 0.)
}

#[system_once]
fn warp_startup_system(
    aspect: &Aspect,
    gpu_interface: &GpuInterface,
    world_render_manager: &mut WorldRenderManager,
    material_test_query: Query<&MaterialTest>,
) {
    let scared_distance = warp_scared_distance(aspect);
    let Some(material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "warp")
    else {
        error!("Could not find warp material test");
        return;
    };
    let Some(Some(material_id)) = material_test.material_id_iter().next() else {
        error!("warp material test is missing expected material_id");
        return;
    };

    let material = gpu_interface
        .material_manager
        .get_material(material_id)
        .unwrap();
    let material_uniforms = material.generate_default_material_uniforms().unwrap();

    world_render_manager.add_or_update_postprocess(material, material_uniforms);

    let arrow_up_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/arrow_up.png".into())
        .unwrap()
        .id();
    let scared_id = gpu_interface
        .texture_asset_manager
        .get_texture_by_path(&"textures/scared.png".into())
        .unwrap()
        .id();

    let mut texture_component_builder = create_new_texture(
        screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.5.into())
            .extend(0.)
            .into(),
        *palette::WHITE,
        arrow_up_id,
        Some(Vec2::splat(aspect.width * 0.08)),
    );
    texture_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&texture_component_builder.build());

    let mut texture_component_builder = create_new_texture(
        scared_distance.extend(0.).into(),
        *palette::WHITE,
        scared_id,
        Some(Vec2::splat(aspect.width * 0.11)),
    );
    texture_component_builder.add_components(bundle_for_builder!(
        MaterialTestObject,
        TimePassedSinceCreation::default()
    ));
    Engine::spawn(&texture_component_builder.build());

    let mut text_component_builder = create_new_text::<_, HeaderText>(CreateTextInput {
        position: screen_space_coordinate_by_percent(aspect, 0.5.into(), 0.7.into()).extend(0.),
        text: "This is up",
        ..Default::default()
    });
    text_component_builder.add_component(MaterialTestObject);
    Engine::spawn(&text_component_builder.build());
    set_system_enabled!(true, warp_system);
}

#[system]
fn warp_system(
    aspect: &Aspect,
    frame_constants: &FrameConstants,
    world_render_manager: &mut WorldRenderManager,
    material_test_query: Query<&MaterialTest>,
    mut texture_query: Query<(&mut Transform, &TextureRender, &mut TimePassedSinceCreation)>,
) {
    let scared_distance = warp_scared_distance(aspect);
    let Some(material_test) = material_test_query
        .iter()
        .find(|material_test| material_test.name() == "warp")
    else {
        error!("Could not find warp material test");
        return;
    };
    let Some(Some(material_id)) = material_test.material_id_iter().next() else {
        error!("warp material test is missing expected material_id");
        return;
    };

    texture_query.for_each(|(transform, _, time_passed_since_creation)| {
        *time_passed_since_creation += frame_constants.delta_time;
        let rotation_matrix = Mat2::from_angle(***time_passed_since_creation);
        transform.position = (rotation_matrix * scared_distance).extend(0.).into();
        transform.rotation += (***time_passed_since_creation).cos() / 8.;
    });

    let current_material_uniforms = &mut world_render_manager
        .get_postprocess_by_material_id_mut(material_id)
        .unwrap()
        .material_uniforms;

    let warp_factor = current_material_uniforms.get("param_0").unwrap();

    let new_value = match warp_factor {
        UniformValue::Array(_) => unreachable!(),
        UniformValue::F32(uniform_var) => {
            let current_value = uniform_var.current_value();
            const INCREMENT_FACTOR: f32 = 0.0005;
            current_value + INCREMENT_FACTOR
        }
        UniformValue::Vec4(_) => unreachable!(),
    };

    current_material_uniforms
        .update("param_0", new_value.into())
        .unwrap();
}

#[derive(Debug, Component, serde::Deserialize, serde::Serialize)]
pub struct FpsCounter;

#[system]
fn fps_system(
    aspect: &Aspect,
    frame_constants: &FrameConstants,
    view: &View,
    mut fps_counters: Query<(&mut TextRender, &FpsCounter)>,
) {
    if matches!(view.view_state(), ViewState::Material((_, _))) {
        let fps_text = format!("FPS: {}", frame_constants.frame_rate);
        if fps_counters.is_empty() {
            let mut text_component_builder = create_new_text::<_, CustomText>(CreateTextInput {
                text: fps_text,
                position: screen_space_coordinate_by_percent(aspect, 0.05.into(), 0.975.into())
                    .extend(4000.),
                text_type: TextTypes::Custom(24.),
                ..Default::default()
            });
            text_component_builder
                .add_components(bundle_for_builder!(MaterialTestObject, FpsCounter));
            Engine::spawn(&text_component_builder.build());
        } else {
            fps_counters.for_each(|(text_render, _)| {
                text_render.text = str_to_u8_array(&fps_text);
            });
        }
    }
}

#[derive(Debug, Component, serde::Deserialize)]
/// Simple [`Component`] for capturing the TextureIds being loaded
pub struct MaterialTextureAsset(TextureId);

impl From<TextureId> for MaterialTextureAsset {
    fn from(value: TextureId) -> Self {
        Self(value)
    }
}

impl MaterialTextureAsset {
    pub fn new(texture_id: TextureId) -> Self {
        Self(texture_id)
    }

    pub fn texture_id(&self) -> &TextureId {
        &self.0
    }
}

#[derive(Debug, Component, serde::Deserialize)]
/// Simple [`Component`] for capturing the TextIds being loaded
pub struct MaterialTextAsset(TextId);

impl From<TextId> for MaterialTextAsset {
    fn from(value: TextId) -> Self {
        Self(value)
    }
}

impl MaterialTextAsset {
    pub fn new(text_id: TextId) -> Self {
        Self(text_id)
    }

    pub fn text_id(&self) -> &TextId {
        &self.0
    }
}

#[derive(Debug, Component, serde::Deserialize)]
/// Simple [`Component`] for capturing the Materials being loaded
pub struct MaterialAsset(MaterialId);

impl From<MaterialId> for MaterialAsset {
    fn from(value: MaterialId) -> Self {
        Self(value)
    }
}

impl MaterialAsset {
    pub fn new(text_id: MaterialId) -> Self {
        Self(text_id)
    }

    pub fn material_id(&self) -> &MaterialId {
        &self.0
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    PartialEq,
    Eq,
    Serialize,
    serde::Deserialize,
    serde::Serialize,
)]
/// A [newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) around [`usize`] to indicate an id for a [`MaterialTest`]
pub struct MaterialTestId(usize);

impl MaterialTestId {
    pub fn increment_id(&self) -> Self {
        Self(&self.0 + 1)
    }
}

impl Deref for MaterialTestId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for MaterialTestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct MaterialIdAlreadySet;

impl Display for MaterialIdAlreadySet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MaterialIdAlreadySet")
    }
}

impl Error for MaterialIdAlreadySet {}
unsafe impl Sync for MaterialIdAlreadySet {}
unsafe impl Send for MaterialIdAlreadySet {}

#[derive(Debug, Component, serde::Deserialize)]
pub struct MaybeLoadedMaterial {
    material_type: MaterialType,
    material_id: Option<MaterialId>,
    text_id: TextId,
}

impl Default for MaybeLoadedMaterial {
    fn default() -> Self {
        Self {
            material_type: MaterialType::Sprite,
            material_id: None,
            text_id: MISSING_TEXT_ID,
        }
    }
}

impl MaybeLoadedMaterial {
    pub fn new(material_type: MaterialType, text_id: TextId) -> Self {
        Self {
            material_type,
            material_id: None,
            text_id,
        }
    }

    pub fn new_material_loaded(material_type: MaterialType, material_id: MaterialId) -> Self {
        Self {
            material_type,
            material_id: Some(material_id),
            text_id: MISSING_TEXT_ID,
        }
    }

    pub fn material_type(&self) -> MaterialType {
        self.material_type
    }

    pub fn material_id(&self) -> Option<MaterialId> {
        self.material_id
    }

    pub fn text_id(&self) -> TextId {
        self.text_id
    }

    pub fn set_material_id(&mut self, material_id: MaterialId) -> Result<(), MaterialIdAlreadySet> {
        if self.material_id.is_some() {
            return Err(MaterialIdAlreadySet);
        }

        self.material_id = Some(material_id);

        Ok(())
    }
}

#[derive(Debug, Component, serde::Deserialize)]
/// A [`Component`] for identifying useful information for running a material
/// test as well as a bool indicating if it is active or not. The intent is that
/// only one `MaterialTest` should be active at a time
pub struct MaterialTest {
    id: MaterialTestId,
    #[serde(with = "BigArray")]
    name: [u8; 256],
    maybe_loaded_materials: [MaybeLoadedMaterial; 25],
    material_type: MaterialType,
    #[serde(with = "BigArray")]
    startup_system_name: [u8; 256],
}

impl MaterialTest {
    pub fn new(
        desired_name: &str,
        startup_system: &CStr,
        maybe_loaded_materials: &[MaybeLoadedMaterial],
        material_type: &MaterialType,
        material_test_id_holder: &mut MaterialTestIdHolder,
    ) -> Self {
        let name = material_test_id_holder.validate_new_name(desired_name);
        Self {
            id: material_test_id_holder.get_next_id(),
            maybe_loaded_materials: array_from_iterator(maybe_loaded_materials.iter().cloned()),
            material_type: *material_type,
            name: str_to_u8_array(name.as_str()),
            startup_system_name: cstr_to_u8_array(startup_system),
        }
    }

    pub fn id(&self) -> MaterialTestId {
        self.id
    }

    pub fn material_id_iter(&self) -> impl Iterator<Item = Option<MaterialId>> + use<'_> {
        self.maybe_loaded_materials
            .iter()
            .map(|maybe_loaded_material| maybe_loaded_material.material_id())
    }

    pub fn material_type(&self) -> &MaterialType {
        &self.material_type
    }

    pub fn name<'b, 'a: 'b>(&'a self) -> &'b str {
        u8_array_to_str(&self.name).unwrap()
    }

    pub fn startup_system_name(&self) -> &CStr {
        u8_array_to_cstr(&self.startup_system_name).unwrap()
    }

    pub fn update_maybe_loaded_materials(&mut self, text_id: TextId, material_id: MaterialId) {
        for maybe_loaded_material in &mut self.maybe_loaded_materials {
            if maybe_loaded_material.text_id() == MISSING_TEXT_ID
                && maybe_loaded_material.material_id().is_none()
            {
                break;
            }

            if maybe_loaded_material.text_id() == text_id
                && maybe_loaded_material.set_material_id(material_id).is_err()
            {
                log::warn!("Attempted to set material that was already set on MaybeLoadedMaterial");
            }
        }
    }
}

/// This is a marker [`Component`] intended to mark assets used in a Material Test that should be cleaned up when changing or clearing material tests
#[derive(Debug, Component, serde::Deserialize)]
pub struct MaterialTestObject;

/// A [`Resource`] for ensuring there are no id clashes with [`MaterialTest`]s
#[derive(Debug, Default, Resource)]
pub struct MaterialTestIdHolder {
    next_id: MaterialTestId,
    taken_test_names: Vec<String>,
}

impl MaterialTestIdHolder {
    pub fn get_next_id(&mut self) -> MaterialTestId {
        let next_id = self.next_id;
        self.next_id = next_id.increment_id();
        next_id
    }

    pub fn validate_new_name(&mut self, desired_name: &str) -> String {
        if self
            .taken_test_names
            .iter()
            .any(|existing_name| existing_name == desired_name)
        {
            self.validate_new_name(format!("{desired_name}0").as_str())
        } else {
            let name = desired_name.to_string();
            self.taken_test_names.push(name.clone());
            name
        }
    }
}

fn wrap_index(index: isize, array_len: usize) -> usize {
    let len = array_len as isize;
    (((index % len) + len) % len) as usize
}

#[system]
fn handle_inputs(
    selectables_query: Query<(&TextRender, &Transform, &Color, &RegularText)>,
    mut underline_query: Query<(&EntityId, &mut Transform, &Color, &Underline)>,
    material_test_query: Query<&MaterialTest>,
    aspect: &Aspect,
    input_state: &InputState,
    view_system: &mut View,
) {
    match view_system.view_state() {
        ViewState::Loading => {
            // no inputs during loading
        }
        ViewState::MainView(material_types) => {
            let left_pressed = is_left_just_pressed(input_state);
            let right_pressed = is_right_just_pressed(input_state);
            let select_pressed = is_select_just_pressed(input_state);

            if select_pressed {
                view_system
                    .set_transition_to(TransitionTo::MaterialSelection(*material_types, None));
                return;
            }

            if left_pressed && right_pressed {
                return;
            }

            if left_pressed || right_pressed {
                let new_material_type = match material_types {
                    MaterialType::Sprite => MaterialType::PostProcessing,
                    MaterialType::PostProcessing => MaterialType::Sprite,
                };

                view_system.view_state = ViewState::MainView(new_material_type);

                selectables_query
                    .iter()
                    .try_for_each(|query_components_ref| {
                        let (text_render, transform, _, _) = query_components_ref.unpack();
                        if u8_array_to_str(&text_render.text).unwrap()
                            == title_from_material_type(&new_material_type)
                        {
                            if let Some(mut components) = underline_query.iter_mut().next() {
                                let (_, underline_transform, _, _) = components.unpack();
                                let underline_offset =
                                    Vec3::new(0., *UNDERLINE_OFFSET_Y_PERCENT * aspect.height, 0.);
                                underline_transform
                                    .position
                                    .set(transform.position.get() - underline_offset);
                                return ControlFlow::Break(());
                            }
                        }

                        ControlFlow::Continue(())
                    });
            }
        }
        ViewState::MaterialSelection((material_type, material_test_id, material_id_order)) => {
            if is_back_just_pressed(input_state) {
                let Some(esc_transition) = view_system.esc_transition else {
                    error!("esc transition must be set in MaterialSelection View");
                    return;
                };
                view_system.set_transition_to(esc_transition);
                return;
            }

            let select_pressed = is_select_just_pressed(input_state);
            if select_pressed && !material_id_order.is_empty() {
                let material_test_id = material_test_id.unwrap();
                view_system
                    .set_transition_to(TransitionTo::Material((*material_type, material_test_id)));
                let material_test = material_test_query
                    .iter()
                    .find(|material_test| material_test.id() == material_test_id)
                    .unwrap();
                Engine::set_system_enabled(material_test.startup_system_name(), true, module_name);
                return;
            }

            let (left_pressed, right_pressed) = {
                let left_pressed = is_left_just_pressed(input_state);
                let right_pressed = is_right_just_pressed(input_state);

                if left_pressed && right_pressed {
                    (false, false)
                } else {
                    (left_pressed, right_pressed)
                }
            };

            let (up_pressed, down_pressed) = {
                let up_pressed = is_up_just_pressed(input_state);
                let down_pressed = is_down_just_pressed(input_state);

                if up_pressed && down_pressed {
                    (false, false)
                } else {
                    (up_pressed, down_pressed)
                }
            };

            if !material_id_order.is_empty()
                && (left_pressed || right_pressed || up_pressed || down_pressed)
            {
                let current_index = material_id_order
                    .iter()
                    .position(|material_test_id_in_vec| {
                        material_test_id_in_vec == &material_test_id.unwrap()
                    })
                    .unwrap();
                let index_shift = if left_pressed {
                    -1
                } else if right_pressed {
                    1
                } else {
                    0
                } + if up_pressed {
                    -2
                } else if down_pressed {
                    2
                } else {
                    0
                };
                let new_index = wrap_index(
                    current_index as isize + index_shift,
                    material_id_order.len(),
                );
                let selected_material_test_id = material_id_order[new_index];

                let selected_material_test_ref = material_test_query
                    .iter()
                    .find(|material_test| material_test.id() == selected_material_test_id);
                let selected_material_test = selected_material_test_ref.unwrap();
                view_system.view_state = ViewState::MaterialSelection((
                    *material_type,
                    Some(selected_material_test_id),
                    material_id_order.clone(),
                ));

                selectables_query
                    .iter()
                    .try_for_each(|query_components_ref| {
                        let (text_render, transform, _, _) = query_components_ref.unpack();
                        if u8_array_to_str(&text_render.text).unwrap()
                            == selected_material_test.name()
                        {
                            if let Some(mut components) = underline_query.iter_mut().next() {
                                let (_, underline_transform, _, _) = components.unpack();
                                let underline_offset =
                                    Vec3::new(0., *UNDERLINE_OFFSET_Y_PERCENT * aspect.height, 0.);
                                underline_transform
                                    .position
                                    .set(transform.position.get() - underline_offset);
                                return ControlFlow::Break(());
                            }
                        }

                        ControlFlow::Continue(())
                    });
            }
        }
        ViewState::Material((material_test_id, material_test_name)) => {
            if is_back_just_pressed(input_state) {
                let Some(esc_transition) = view_system.esc_transition else {
                    error!(
                        "Esc transition not set from material test {material_test_id} {material_test_name}. This is an error"
                    );
                    return;
                };
                view_system.set_transition_to(esc_transition);
            }
        }
    }
}

#[system_once]
fn view_system(
    interactive_text_query: Query<(&EntityId, &InteractiveText)>,
    noninteractive_text_query: Query<(&EntityId, &NonInteractiveText)>,
    mut material_test_query: Query<&mut MaterialTest>,
    material_test_object_query: Query<(&EntityId, &MaterialTestObject)>,
    aspect: &Aspect,
    view_handler: &mut View,
    world_render_manager: &mut WorldRenderManager,
) {
    view_handler.change_view(
        &interactive_text_query,
        &noninteractive_text_query,
        &mut material_test_query,
        &material_test_object_query,
        aspect,
        world_render_manager,
    );
}

// Marker Components for Text

#[derive(Debug, Component, serde::Deserialize)]
pub struct HeaderText;

#[derive(Debug, Component, serde::Deserialize)]
pub struct RegularText;

#[derive(Debug, Component, serde::Deserialize)]
pub struct CustomText;

#[derive(Debug, Component, serde::Deserialize)]
pub struct Underline;

#[derive(Debug, Component, serde::Deserialize)]
pub struct NonInteractiveText;

#[derive(Debug, Component, serde::Deserialize)]
pub struct InteractiveText(TransitionTo);

#[derive(Debug, Component, serde::Deserialize, serde::Serialize)]
pub struct TimePassedSinceCreation(f32);

impl Default for TimePassedSinceCreation {
    fn default() -> Self {
        Self(0.)
    }
}

impl Deref for TimePassedSinceCreation {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add<f32> for TimePassedSinceCreation {
    type Output = TimePassedSinceCreation;

    fn add(self, right_hand_side: f32) -> Self::Output {
        let time_passed_value = if (f32::MAX - right_hand_side) < right_hand_side {
            0.
        } else {
            self.0 + right_hand_side
        };
        TimePassedSinceCreation(time_passed_value)
    }
}

impl AddAssign<f32> for &mut TimePassedSinceCreation {
    fn add_assign(&mut self, right_hand_side: f32) {
        **self = **self + right_hand_side;
    }
}

impl InteractiveText {
    pub fn new(transition_to: TransitionTo) -> Self {
        Self(transition_to)
    }
}

impl Deref for InteractiveText {
    type Target = TransitionTo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
/// State Machine for Handling the Intended State of the Main View
///
/// * [`ViewState::Loading`] happens before the entry point while assets load
/// * [`ViewState::MainView`] is the intended entry point, should display the different [`MaterialType`]s
/// * [`ViewState::MaterialSelection`] is a selection view of tests grouped under the selected [`MaterialType`]s
/// * [`ViewState::Material`] should display the selected Material Test
pub enum ViewState {
    #[default]
    Loading,
    MainView(MaterialType),
    /// The middle enum value is an optional selection of a starting MaterialTest.id and the last enum value is a list of all possible MaterialTest ids for the selected [`MaterialType`]
    MaterialSelection((MaterialType, Option<MaterialTestId>, Vec<MaterialTestId>)),
    Material((MaterialTestId, String)),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, serde::Deserialize)]
pub enum TransitionTo {
    #[default]
    Loading,
    MainView,
    MaterialSelection(MaterialType, Option<MaterialTestId>),
    Material((MaterialType, MaterialTestId)),
}

#[derive(Debug, Resource)]
pub struct View {
    transitioning_to: Option<TransitionTo>,
    view_state: ViewState,
    pub esc_transition: Option<TransitionTo>,
    pub post_load_transition: Option<TransitionTo>,
}

impl Default for View {
    fn default() -> Self {
        Self {
            transitioning_to: Some(TransitionTo::default()),
            view_state: ViewState::default(),
            esc_transition: None,
            post_load_transition: None,
        }
    }
}

impl View {
    pub fn view_state(&self) -> &ViewState {
        &self.view_state
    }

    pub fn clear_transitioning_to(&mut self) {
        self.transitioning_to = None;
    }

    pub fn get_transitioning_to(&self) -> Option<&TransitionTo> {
        self.transitioning_to.as_ref()
    }

    pub fn set_transition_to(&mut self, new_transitioning_to: TransitionTo) {
        self.transitioning_to = Some(new_transitioning_to);
        set_system_enabled!(true, view_system);
    }

    pub fn change_view(
        &mut self,
        interactive_text_query: &Query<(&EntityId, &InteractiveText)>,
        noninteractive_text_query: &Query<(&EntityId, &NonInteractiveText)>,
        material_test_query: &mut Query<&mut MaterialTest>,
        material_test_object_query: &Query<(&EntityId, &MaterialTestObject)>,
        aspect: &Aspect,
        world_render_manager: &mut WorldRenderManager,
    ) {
        let Some(ref transition_to) = self.transitioning_to else {
            error!(
                "change_view function was triggered without a transitioning_to state set, this should not happen"
            );
            return;
        };

        noninteractive_text_query.iter().for_each(|query_ref| {
            let (entity_id, _) = query_ref.unpack();
            Engine::despawn(**entity_id);
        });
        interactive_text_query.iter().for_each(|query_ref| {
            let (entity_id, _) = query_ref.unpack();
            Engine::despawn(**entity_id);
        });
        material_test_object_query
            .iter()
            .for_each(|material_test_object_query_ref| {
                let (entity_id, _) = material_test_object_query_ref.unpack();
                Engine::despawn(**entity_id);
            });

        match transition_to {
            TransitionTo::Loading => {
                self.esc_transition = None;

                let mut text_component_builder =
                    create_new_text::<_, HeaderText>(CreateTextInput {
                        text: "Loading...",
                        text_type: TextTypes::Header,
                        position: screen_space_coordinate_by_percent(
                            aspect,
                            0.5.into(),
                            0.5.into(),
                        )
                        .extend(0.),
                        ..Default::default()
                    });
                text_component_builder.add_component(NonInteractiveText);
                Engine::spawn(&text_component_builder.build());
            }
            TransitionTo::MainView => {
                self.esc_transition = None;

                turn_off_material_test_systems();

                let postprocess_material_ids = world_render_manager
                    .postprocesses()
                    .iter()
                    .map(|post_process| *post_process.material_id())
                    .collect::<Vec<_>>();
                world_render_manager.remove_postprocesses(&postprocess_material_ids);

                let mut text_component_builder =
                    create_new_text::<_, HeaderText>(CreateTextInput {
                        text: "Choose Material Type:",
                        text_type: TextTypes::Header,
                        position: screen_space_coordinate_by_percent(
                            aspect,
                            0.5.into(),
                            0.75.into(),
                        )
                        .extend(0.),
                        ..Default::default()
                    });
                text_component_builder.add_component(NonInteractiveText);
                Engine::spawn(&text_component_builder.build());

                let standard_material_text_position =
                    screen_space_coordinate_by_percent(aspect, 0.25.into(), 0.60.into()).extend(0.);
                let mut text_component_builder =
                    create_new_text::<_, RegularText>(CreateTextInput {
                        text: title_from_material_type(&MaterialType::Sprite),
                        text_type: TextTypes::Regular,
                        position: standard_material_text_position,
                        ..Default::default()
                    });
                text_component_builder.add_component(InteractiveText::new(
                    TransitionTo::MaterialSelection(MaterialType::Sprite, None),
                ));
                Engine::spawn(&text_component_builder.build());

                let mut text_component_builder =
                    create_new_text::<_, RegularText>(CreateTextInput {
                        text: title_from_material_type(&MaterialType::PostProcessing),
                        text_type: TextTypes::Regular,
                        position: screen_space_coordinate_by_percent(
                            aspect,
                            0.75.into(),
                            0.60.into(),
                        )
                        .extend(0.),
                        ..Default::default()
                    });
                text_component_builder.add_component(InteractiveText::new(
                    TransitionTo::MaterialSelection(MaterialType::PostProcessing, None),
                ));
                Engine::spawn(&text_component_builder.build());

                self.view_state = ViewState::MainView(MaterialType::Sprite);

                let underline_offset =
                    Vec3::new(0., *UNDERLINE_OFFSET_Y_PERCENT * aspect.height, 0.);
                let mut underline_component_builder = create_underline(
                    (standard_material_text_position - underline_offset).into(),
                    None,
                    aspect,
                );
                underline_component_builder.add_component(NonInteractiveText);
                Engine::spawn(&underline_component_builder.build());
            }
            TransitionTo::MaterialSelection(material_type, specified_material_test_id) => {
                self.esc_transition = Some(TransitionTo::MainView);

                turn_off_material_test_systems();

                let postprocess_material_ids = world_render_manager
                    .postprocesses()
                    .iter()
                    .map(|post_process| *post_process.material_id())
                    .collect::<Vec<_>>();
                world_render_manager.remove_postprocesses(&postprocess_material_ids);

                let mut text_component_builder =
                    create_new_text::<_, HeaderText>(CreateTextInput {
                        text: title_from_material_type(material_type),
                        text_type: TextTypes::Header,
                        position: screen_space_coordinate_by_percent(
                            aspect,
                            0.5.into(),
                            0.75.into(),
                        )
                        .extend(0.),
                        ..Default::default()
                    });
                text_component_builder.add_component(NonInteractiveText);
                Engine::spawn(&text_component_builder.build());

                let mut material_test_id_order = vec![];
                let left_column_starting_position =
                    screen_space_coordinate_by_percent(aspect, 0.25.into(), 0.6.into()).extend(0.);
                let right_column_starting_position =
                    screen_space_coordinate_by_percent(aspect, 0.75.into(), 0.6.into()).extend(0.);
                material_test_query
                    .iter()
                    .filter(|material_test| material_test.material_type() == material_type)
                    .enumerate()
                    .for_each(|(index, material_test)| {
                        material_test_id_order.push(material_test.id);

                        let (quotient, remainder) = division_result(index, 2);
                        let position = if remainder % 2 == 0 {
                            left_column_starting_position
                        } else {
                            right_column_starting_position
                        } - quotient as f32 * Vec3::new(0., 0.1 * aspect.height, 0.);

                        let mut text_component_builder =
                            create_new_text::<_, RegularText>(CreateTextInput {
                                text: u8_array_to_str(&material_test.name).unwrap(),
                                text_type: TextTypes::Regular,
                                position,
                                ..Default::default()
                            });

                        text_component_builder.add_component(InteractiveText::new(
                            TransitionTo::Material((*material_type, material_test.id)),
                        ));
                        Engine::spawn(&text_component_builder.build());

                        let should_add_underline =
                            if let Some(specified_material_test_id) = specified_material_test_id {
                                specified_material_test_id == &material_test.id
                            } else {
                                index == 0
                            };
                        if should_add_underline {
                            let underline_offset =
                                Vec3::new(0., *UNDERLINE_OFFSET_Y_PERCENT * aspect.height, 0.);
                            let mut underline_component_builder = create_underline(
                                (position - underline_offset).into(),
                                None,
                                aspect,
                            );
                            underline_component_builder.add_component(NonInteractiveText);
                            Engine::spawn(&underline_component_builder.build());
                        }
                    });

                self.view_state = ViewState::MaterialSelection((
                    *material_type,
                    if let Some(specified_material_test) = specified_material_test_id {
                        Some(*specified_material_test)
                    } else {
                        material_test_id_order.first().copied()
                    },
                    material_test_id_order,
                ));
            }
            TransitionTo::Material((material_type, material_test_id)) => {
                if material_test_query.is_empty() {
                    return;
                }
                self.esc_transition = Some(TransitionTo::MaterialSelection(
                    *material_type,
                    Some(*material_test_id),
                ));

                let name = material_test_query
                    .iter()
                    .find(|material_test_object| material_test_object.id() == *material_test_id)
                    .unwrap()
                    .name()
                    .to_string();
                self.view_state = ViewState::Material((*material_test_id, name));
            }
        }
        self.clear_transitioning_to();
    }
}

// This includes auto-generated C FFI code (saves you from writing it manually).
include!(concat!(env!("OUT_DIR"), "/ffi.rs"));

#[cfg(test)]
mod test {
    use game_asset::{
        ecs_module::MaterialManager,
        resource_managers::material_manager::{DEFAULT_SHADER_ID, DEFAULT_SHADER_TEXT},
    };

    use crate::test_validation::WgslValidator;

    #[test]
    fn validate_shader() {
        let invalid_wgsl = DEFAULT_SHADER_TEXT;
        let mut validation = WgslValidator::default();
        assert!(validation.validate_wgsl_string(invalid_wgsl).is_err());
        let mut material_manager = MaterialManager::default();
        let color_replacement_toml_string =
            include_str!("../assets/toml_materials/sprite/color_replacement.toml");
        let color_replacement_material_id = material_manager
            .register_material_from_string(
                DEFAULT_SHADER_ID,
                "color_replacement",
                color_replacement_toml_string,
            )
            .unwrap();
        let valid_wgsl = material_manager
            .generate_shader_text(color_replacement_material_id)
            .unwrap();
        validation.validate_wgsl_string(&valid_wgsl).unwrap();
        let wgsl_metadata = validation.emit_wgsl_metadata(&valid_wgsl).unwrap();
        assert_eq!(
            wgsl_metadata.types_iter().collect::<Vec<&str>>(),
            vec![
                "GlobalUniforms",
                "SceneInstance",
                "VertexInput",
                "VertexOutput"
            ]
        );
        assert_eq!(
            wgsl_metadata.constants_iter().collect::<Vec<&str>>(),
            Vec::<&str>::new()
        );
        assert_eq!(
            wgsl_metadata.special_types_iter().collect::<Vec<&str>>(),
            Vec::<&str>::new()
        );
        assert_eq!(
            wgsl_metadata.overrides_iter().collect::<Vec<&str>>(),
            Vec::<&str>::new()
        );
        assert_eq!(
            wgsl_metadata.global_variables_iter().collect::<Vec<&str>>(),
            vec![
                "global_uniforms",
                "scene_instances",
                "scene_indices",
                "color_tex",
                "sampler_color_tex"
            ]
        );
        assert_eq!(
            wgsl_metadata.functions_iter().collect::<Vec<&str>>(),
            vec!["get_world_offset", "get_fragment_color"]
        );
        assert_eq!(
            wgsl_metadata.entry_points_iter().collect::<Vec<&str>>(),
            vec!["vs_main", "fs_main"]
        );
    }

    #[ignore]
    #[test]
    // This is a helper function for outputing the shader string while developing a shader
    fn output_shader_string() {
        let mut material_manager = MaterialManager::default();
        let toml_string = include_str!("../assets/toml_materials/sprite/scrolling_color.toml");
        let material_id = material_manager
            .register_material_from_string(DEFAULT_SHADER_ID, "scrolling_color", toml_string)
            .unwrap();
        let wgsl = material_manager.generate_shader_text(material_id).unwrap();
        println!("{wgsl}");
        // Failing this test otherwise std out is supressed
        panic!();
    }
}
