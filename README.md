# Shader Test Module

This module demonstrates Fiasco's Material System. The Material System gives developers a structured means of developing [shaders](https://en.wikipedia.org/wiki/Shader) which can be orchestrated by our ECS system. A Material, much like materials in other engines such as [Unity](https://docs.unity3d.com/Manual/Materials.html), [Unreal](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-materials) and [Godot](https://docs.godotengine.org/en/stable/classes/class_material.html), is a collection of code, data structure definitions, and texture definitions defining how something should be rendered by a graphics pipeline. In our system specifically, it is a Shader Template, a Material Definition, and a set of Material Parameters for that Material definition. If this feels like a bit of a jargon soup at this point, that's okay, we're going to walk through creating a material and go over each of these terms in depth. You can use this living module to modify and experiment with the existing materials yourself, or try to make your own materials in an isolated environment.

## Prerequisites

With this guide, we'll make a Sprite Material together. We do presume some knowledge of general programming practices, Rust, and WGSL when using this guide. We have set up easy automation for running this project in VSCode. Other IDEs or editors can be used, but they will require setup that isn't discussed in this guide.

* Clone the [Engine repo](https://github.com/vaguevoid/engine), install its prerequisites, and run `cargo build`
* Clone [cargo-fiasco](https://github.com/vaguevoid/cargo-fiasco), build it, and be sure to set up cargo fiasco on your PATH (there should be instructions in its README)
* Modify the .vscode folder of this repo for yourself locally, both `launch.json` and `tasks.json` need their paths modified to your local engine folder. For example, if your engine folder is in `/Users/shaderguru/repos/engine`, the path you need to insert on macOS or Linux is `/Users/shaderguru/repos/engine/target/debug/platform_native`. You can use the Run and Debug icon in the VSCode side panel to launch the engine with this module.

After the module is started, you can use the arrow keys to navigate to different options, Return to select options, and the Esc key to go back. Now let's look at how we'd develop a color replacement Material with this series of APIs.

## Developing a Material

### Shader Template

#### Shader Template Quick Overview

* Select your Shader Template
  * Do you want to apply your material to a quad or sprite? Use the Sprite Material Shader Template: `DEFAULT_SHADER_ID`
  * Do you want to apply your material to the entire scene after rendering? Use the Post Processing Shader Template: `DEFAULT_POST_PROCESSING_SHADER_ID`.

#### Shader Template Detailed Explanation

First, we must select a shader template. The two primary shader templates are in the Engine repo, in `crates/game_asset/shaders/shader_templates`. A material that will be applied to a renderable entity (likely the most common use case) uses the default or Sprite Material shader template. A material to be applied to the entire, generated image after we've rendered everything, would use the post_processing_default, aka the Post Processing Material template. We will be developing a color replacement shader, so we'll use the default shader since this will be a "sprite" material.

In a typical use case, this is the only decision you will have to make regarding Shader Templates. One outstanding thing we haven't decided on is how we will allow users to share and inject custom WGSL functions, but once we've decided that this README will be updated with an example.

### Material Definition

#### Material Definition Quick Overview

* Define your material in a TOML file
  * See examples in this repo or engine repo at `crates/game_asset/toml_shaders`
* The TOML keys are...
  * `get_world_offset`
    * Required
    * String, function body for vertex shader
    * Must be valid WGSL
  * `get_fragment_color`
    * Required
    * String, function body for fragment shader
    * Must be valid WGSL
  * `uniform_types`
    * Optional
    * map of keys to uniform types
      * Valid uniform types are
        * `f32`
        * `vec4f`
        * `array<vec4f, N>`, where N is the size of the array
      * `my_uniform_name = "f32"` OR `my_uniform_name = { type: "f32", default: "8.8" }` valid
    * Generates uniforms in a struct in WGSL code called `dynamic_uniforms`
  * `texture_descs`
    * Optional
    * map of texture names to texture filter mode
      * `nearest` means direct sampling from texture, `linear` means interpolated
      * `my_texture_name = "filter"` valid
* Create material by calling `MaterialManager`::`register_material_from_string`
* Create material pipeline by calling `PipelineManager`::`register_pipeline`

#### Material Definition Detailed Explanation

A Material is a data structure that defines the expected behavior of a shader, and any inputs the shader accepts. We can define how the shader will manipulate a sprite's vertices, how it will define the color at every pixel, define any data variables from the CPU we want to feed into the shader/GPU (called [uniforms](https://thebookofshaders.com/03/) in shaderland), and any textures we'll want to [sample](https://vfxdoc.readthedocs.io/en/latest/textures/sampling/) from.

We use toml files to define our materials. If you are unfamiliar with TOML, check out [this link](https://toml.io/en/), but essentially it is data serialization language like JSON or YAML. Our Material Definition tomls accept 4 keys

* **get_world_offset** - This is required, and this must be a string, and it must be a valid WGSL function body. This is injected into our vertex offset function, which for the default shader template would look like this in context...

```wgsl
fn get_vertex_offset(uv0: vec2f,) -> vec2f {
    // this is the starting point for where get_world_offset would be injected
    return vec2f(0., 0.); // This particular example does nothing
    //this is the end point for where get_world_offset would be injected
}
```

For this and `get_fragment_color`, uv0 means the [uv coordinates](https://en.wikipedia.org/wiki/UV_mapping).

* **get_fragment_color** - This is required, and this must be a string, and it must be a valid WGSL function body. This is injected into our fragment color function, which for the default shader template would look like this in context...

```wgsl
fn get_fragment_color(uv0: vec2f, vertex_color: vec4f) -> vec4f {
    // this is the starting point for where get_fragement_color would be injected
    return vertex_color; // This particular example does nothing
    //this is the end point for where get_fragement_color would be injected
}
```

* **uniform_types** - This is optional, and is a map. To see some examples, feel free to look in this repo in `assets/toml_materials` or in the engine repo in `game_asset/shaders/toml_shaders`. The keys are strings, and the values can be one of two types. It can be a string that must be one of "vec4f", "array<vec4f, N>" where N is an integer greater than 0, or "f32". It can also be an object with two mandatory keys, the first key is "type" and must be one of the aforementioned values, and the default key must be of the data type specified in type. This is used by `MaterialManager` to generate a WGSL struct. To pass a value into your shader that is a `f32` with the name `my_custom_value`, you'd include a member in the map of `my_custom_value = "f32"` or `my_custom_value = { type = "f32", default = "32.23" }`

* **texture_descs** - This is optional, and is a map. The keys are strings, and the value is either "nearest" or "filter" which represents the [filter mode](https://registry.khronos.org/vulkan/specs/latest/html/vkspec.html#textures-texel-filtering) for the texture. Nearest means the GPU will sample the color from the nearest [texel](https://en.wikipedia.org/wiki/Texel_(graphics)) to the coordinate you are sampling from, and linear means the sample will interpolate between the nearest texels. The correct mode depends on your use case and what you are sampling. `texture_descs` is used by `MaterialManager` to generate WGSL structs and samplers.

Now, let's write shader code and define some uniforms. We will do this in a material definition file. In `assets/toml_materials` create a file called `color_replacement.toml`. In the file, create the `get_world_offset` key. Because the vertices don't need to change, we'll only need a passthrough function body.

```toml
get_world_offset = """
return vec2f(0., 0.);
"""
```

Next up are uniform values for this material. We must specify the color to replace, and the color we will be replacing it with. At a high level, uniforms are the "bridge" values from CPU code into GPU shaders. Specifically for Fiasco, we will be able to pass data into our materials from ECS systems via uniforms. By default, the replacement and insert colors will be the same, so initially this material will effectively do nothing until we modify the uniforms in our system code later in this tutorial.

```toml
[uniform_types]
color_to_replace = { type = "vec4f", default = [0.0, 1.0, 0.0, 1.0] }
color_to_insert = { type = "vec4f", default = [0.0, 1.0, 0.0, 1.0] }
```

Next we'll set up the texture we are going to sample our colors from. We will specify the linear filter mode to smooth out the color values between the texels.

```toml
[texture_descs]
color_tex = "linear"
```

Next we'll set up our fragment shader function body in the `get_fragment_color` key. Note due to design decisions for toml, this key will have to go above `[uniform_types]`, or any other key that doesn't use toml's `key=value` syntax. In our fragment shader code, the uniforms we specified above will be used to replace the color. The uniforms we specify above generate a struct called `dynamic_uniforms` for easy use in our shader code. In addition, we will sample from the texture we defined above. The `MaterialManager` automatically injected `color_tex` and a sampler as `sampler_color_tex`.

```toml
get_fragment_color = """
let sprite_color = textureSample(color_tex, sampler_color_tex, uv0.xy);
if (abs(length(sprite_color - dynamic_uniforms.color_to_replace)) < 0.01) {
    return dynamic_uniforms.color_to_insert;
}
return sprite_color;
"""
```

As you are developing your material, you may be unsure if you are generating wgsl with proper syntax. When you go to add the material to the `MaterialManager` the engine will panic and give you an error message relating to your syntax error. If you don't want to start the whole module or engine every time you iterate, you can use the test in `lib.rs` of this module to quickly iterate on changes to your material definition toml file. There is also an ignored test in `lib.rs` you can manually run to easily print out the `WGSL` code you are generating if that is useful to your development.

We now have a fully defined material. Now we can generate the material in our `MaterialManager`. In this repo, we do that in the system called `material_setup` in `lib.rs`. This function in turn uses the helper functions `register_post_sprite_material` and `register_post_processing_material` in `src/asset_registering`. We will review how those functions work so that you can set up shaders in your own module. The following code registers the material. For the following, `DEFAULT_SHADER_ID` comes from the `engine/game_assets` repo, and is the id associated with the sprite Shader Template. Postprocessing shaders also have a corresponding shader id, which is `DEFAULT_POST_PROCESSING_SHADER_ID`.

```rust
let name = "color_replacement";
let toml_string = include_str!("path/to/color_replacement.toml");
let color_replacement_material_id = material_manager.register_material_from_string(DEFAULT_SHADER_ID, name, toml_string).unwrap();
```

Finally, we'll need to create a `MaterialPipeline` using our `PipelineManager`. Deep details about `PipelineManager` are outside this tutorial's scope, but every material needs a corresponding `MaterialPipeline`. The following code would create one for our new material.

```rust
// We need to access the texture manager. Currently the only way to do this is with the gpu_resource, but this is actually an anti-pattern for our engine. In the future we will fix this, but know that doing this violates our platform/core library architecture and can lead to problems
let texture_manager = &mut gpu_resource.texture_manager;
// This is the texture we will be targeting our output to
let resolve_target = texture_manager.get_render_target(RenderTargetType::ColorResolve);
pipeline_manager.register_pipeline(
    color_replacement_material_id,
    resolve_target.texture.format(),
    1,
    &gpu_resource.device,
    material_manager,
);
```

### Material Parameters

#### Material Parameters Quick Overview

* Generate `MaterialParameters` component by passing in `MaterialId` for your material
* Set your `MaterialParameters` uniforms and textures using `update_uniform_buffer` and `update_texture_asset_ids` functions respectively on your `Material`
* Spawn your entity with this `MaterialParameters` component
  * For your entity to be renderable, it must also have the `TextureRender`, `Color` and `Transform` components
* Updating the uniforms or textures of an entity's `MaterialParameters` with the same `update_uniform_buffer` or `update_texture_asset_ids` will modification those values immediately for that entity's next render pass

#### Material Parameters Detailed Explanation

We are now at the last stage, where we will glue together CPU side code in `system`s to the `Material`s. For this, we will use `MaterialParameters`. `MaterialParameters` are `Component`s that allow us to specify values for the uniforms and textures, and to attach these to individual renderable entities. Currently, a renderable entity is any entity that has the `TextureRender`, `Transform`, and `Color` components.

In this particular repo, there is a lot of boilerplate to set up different examples of materials. We will ignore most of that, and just set up a `color_replacement_system`. The function definition of the system will look like...

```rust
#[system]
fn color_replacment_system(
    aspect: &Aspect,
    frame_constants: &FrameConstants,
    view: &View,
    mut material_test_query: Query<&mut MaterialTest>, // Boilerplate for determining if the color replacement example is active
    mut textures: Query<(
        &TextureRender,
        &mut TimePassedSinceCreation,
        &mut MaterialParameters,
    )>,
    material_manager: &MaterialManager,
    material_test_assets: &MaterialTestAssets, // Resource defined in this repo for managing different material tests
) {
    // We will fill in this body
}
```

This system sets up the resources and queries we'll need to demonstrate the color replacement `Material` with a `MaterialParameter` `Component`. First, in the body of this system, we'll use a boilerplate function to determine if the test was just activated, or if the test is actively running.

```rust
let test_name = "color_replacement";
if let Some((test_just_turned_active, material_ids)) =
    is_material_test_currently_active(test_name, &mut material_test_query, &view.view_state)
{
    let material_id = material_ids[0].1;
    let material = material_manager.get_material(&material_id).unwrap();

    if test_just_turned_active {
        // Setup test
    } else {
        // Do stuff every frame the test is active
    }
}
```

`is_material_test_currently_active` looks to see if `color_replacement` is the active test, and gives you a bool `test_just_turned_active` to let you do setup code versus the code you want to do every frame in the test. Now let's fill in code where we have the comment `// Setup test`.

```rust
let half_width = aspect.width / 2.;
let half_height = aspect.height / 2.;
let mut material_params = MaterialParameters::new(material_id);
let white_color_uniform = Vec4::new(1., 1., 1., 1.).into();
let grey_color_uniform = Vec4::new(0.3, 0.3, 0.3, 1.).into();

let scared_metadata = material_test_assets.get("scared").unwrap();
```

`half_width` and `half_height` are just used for placing the "sprite" that we'll apply the material to. In this example, we'll look for white in our texture and replace it with gray, and `white_color_uniform` and `grey_color_uniforms` respectively represent these values. Then, we are going to replace the color in the beautiful `scared` image found in this repo's assets.

Next, we'll create our `MaterialParameters` `Component`, which is just done by passing in the corresponding `material_id`. The `MaterialParameters` `Component` keeps track of our uniform buffer and texture ids that we'll eventually pass into our GPU code.

The current API design of `MaterialParameters` is not to directly modify `MaterialParameters`, but to use functions on `Material` to correctly set your `MaterialParameters`. Let's go ahead and set up the `color_to_replace` uniform to have the `white_color_uniform` value.

```rust
material
    .update_uniform_buffer(
        "color_to_replace",
        &white_color_uniform,
        &mut material_params.data,
    )
    .unwrap();
```

We'll get the `Material` from the `MaterialManager`, and then we'll call the `update_uniform_buffer` function to modify the `MaterialParameters` safely. The first function parameter is the uniform we want to update, the second parameter is the value we want to update the uniform to, and the last value is the uniform buffer on `MaterialParameters` that will be modified. We are `unwrap`ing here, but if you are unsure about the source of your data you may want to properly handle the `Result` value from `update_uniform_buffer`. If you input the wrong uniform type for `color_to_replace` (in this case either f32 or array<f32; N> would be wrong), the `Result` enum will return the `Error` variant.

Let's set the `color_to_insert` uniform the same way.

```rust
material
    .update_uniform_buffer(
        "color_to_insert",
        &grey_color_uniform,
        &mut material_params.data,
    )
    .unwrap();
```

Now we'll set up our scared texture to make our texture description for `color_tex`. The method is very similar to how we specified our uniform values.

```rust
material
    .update_texture_asset_ids(
        "color_tex",
        *scared_metadata.asset_id(),
        &mut material_params.textures,
    )
    .unwrap();
```

The main differences here are that we are calling `update_texture_asset_ids` on the `Material` instead of `update_uniform_buffer`, which needs the `AssetId` for the texture. We need to pass in the `MaterialParameters` textures array.

Now we need to spawn in a renderable entity and attach this `MaterialParameters` component to it. This repo has a helper function for just this purpose.

```rust
let mut texture_component_builder = create_new_texture(
    Vec3::new(half_width, half_height, 1.), // This is the position on the Transform component
    Vec4::new(1., 1., 1., 1.), // This is the Color. We do not use this in our setup, but must be set for this to be rendered
    *scared_metadata.asset_id(), // Likewise, this is actually not used here but is necessary for the TextureRender component, which in turn is needed to Render this
    Some(Vec2::splat(400.)), // This scales up our sprite to be a bit easier to see
);
// texture_component_builder is a convenient pattern for us to add components before spawning
texture_component_builder.add_components(bundle_for_builder!(
    MaterialTestObject,
    material_params,
    TimePassedSinceCreation::default()
));
Engine::spawn(&texture_component_builder.build());
```

Now we are using our color replacement `Material`, via the `MaterialParameters` `Component`, to replace the white color in the scared image with a gray.

Lastly though let's see how modifying `MaterialParameters` every frame might look like. We're now going to fill in the else statement where we had the comment `// Do stuff every frame the test is active`.

```rust
textures.for_each(|(_, time_passed_since_creation, material_params)| {
    *time_passed_since_creation += frame_constants.delta_time;
    let new_target_color = Vec4::new(
        f32::sin(***time_passed_since_creation * 0.1).abs(),
        f32::cos(***time_passed_since_creation * 0.5).abs(),
        f32::sin(***time_passed_since_creation * 0.3 + 0.5).abs(),
        1.,
    )
    .into();
    material
        .update_uniform_buffer(
            "color_to_insert",
            &new_target_color,
            &mut material_params.data,
        )
        .unwrap();
});
```

We loop through the textures query, even though here there is only one texture that should match that query. We're going to change the target color slightly every frame to briefly touch on the dynamic power of `MaterialParameters`. `TimePassedSinceCreation` is a helper struct that will allow us to use time as a parameter in trigonometric functions to create random feeling color variations in the new target color. We simply create the new color, and once again use the material manager to update the `MaterialParameter`s uniform data. By running this example, you can validate yourself that the replaced color slowly changes with time.

## Implementation Details

Here we will briefly review some implementation details for this system

### Material Definition Implementation Details

We use the combination of Shader Template and information defined in the toml material definitions to generate a WGSL shader string. The `get_world_offset` and `get_fragment_color` are fairly straightforward, these are just inserted into the appropriate place in the Shader Template. The `uniform_types` values are used to generate a `DynamicUniform` struct in the shader. We also arrange these datatypes consistently so that we will be able to manage how the user will pass uniform data from gameplay code into the GPU uniform buffers. Finally, `texture_descs` values are used to generate texture samplers in the shader code.

### Material Parameters Implementation Details

Let's look at the `MaterialParameters` struct definition.

```rust
#[repr(C)]
#[derive(Component, Debug, Pod, Zeroable)]
pub struct MaterialParameters {
    pub material_id: MaterialId,
    pub data: [f32; UNIFORM_LIMIT],
    pub textures: [AssetId; 16],
}
```

Essentially, a `MaterialParameters` holds data ready for the GPU to consume. Most of the guardrails actually come from the helper functions for modifying `MaterialParameters` that live in the method implementations attached to a struct, specifically `update_uniform_buffer` and `update_texture_asset_ids`. Those methods have several layers of checks to ensure that users of these functions don't inadvertently create a uniform buffer that has data in the wrong place. Behind the scenes in `DrawList` and `MaterialPipeline`, these buffers are passed straight into wgpu's graphics APIs, and debugging malformed uniform data can be time consuming and annoying.  The intent of these APIs is to ensure the user won't need to do this sort of debugging themselves.

We have a `Uniforms` enum in the Engine to give us control around the uniform data we will eventually pass to the GPU. The potential source data types for the `Uniforms` enum in Rust, which are `f32`, `Vec4` and `FixedSizeVec<Vec4>`, all have the `Into` trait implemented on them for converting the base data types into `Uniforms`. `FixedSizeVec<T>` is a runtime, fixed size array and corresponds to the gpu data type of `array<vec4, N>`.
