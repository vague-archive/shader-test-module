//! Utility functions related to loading assets, in this case materials and textures.

use std::ffi::CStr;

use game_asset::{
    ecs_module::{GpuInterface, TextAssetManager},
    resource_managers::material_manager::materials::MaterialType,
};
use void_public::{AssetPath, Engine, EventWriter, bundle, event::graphics::NewText, text::TextId};

use crate::{
    MaterialTest, MaterialTestId, MaterialTestIdHolder, MaterialTextAsset, MaybeLoadedMaterial,
};

#[allow(clippy::too_many_arguments)]
pub fn register_material(
    name: &str,
    material_type: MaterialType,
    material_definition_path: &AssetPath,
    startup_system: &CStr,
    gpu_interface: &mut GpuInterface,
    material_test_id_holder: &mut MaterialTestIdHolder,
    event_writer: &EventWriter<NewText<'_>>,
    text_asset_manager: &mut TextAssetManager,
) -> (TextId, MaterialTestId) {
    let pending_text = gpu_interface
        .material_manager
        .load_material_from_path(
            material_type.into_shader_template_id(),
            name,
            material_definition_path,
            true,
            event_writer,
            text_asset_manager,
        )
        .unwrap();
    let material_test = &MaterialTest::new(
        name,
        startup_system,
        &[MaybeLoadedMaterial::new(material_type, pending_text.id())],
        &material_type,
        material_test_id_holder,
    );
    Engine::spawn(bundle!(material_test));
    Engine::spawn(bundle!(&MaterialTextAsset::new(pending_text.id())));

    (pending_text.id(), material_test.id())
}
