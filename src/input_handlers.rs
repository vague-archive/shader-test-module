//! Utility functions related to handling keyboard and mouse inputs.

use void_public::{
    event::input::{KeyCode, MouseButton},
    input::InputState,
};

pub fn any_keys_just_pressed(input_state: &InputState, keys: &[KeyCode]) -> bool {
    keys.iter()
        .any(|key_code| input_state.keys[*key_code].just_pressed())
}

pub fn is_left_just_pressed(input_state: &InputState) -> bool {
    any_keys_just_pressed(input_state, &[KeyCode::ArrowLeft, KeyCode::KeyA])
}

pub fn is_right_just_pressed(input_state: &InputState) -> bool {
    any_keys_just_pressed(input_state, &[KeyCode::ArrowRight, KeyCode::KeyD])
}

pub fn is_up_just_pressed(input_state: &InputState) -> bool {
    any_keys_just_pressed(input_state, &[KeyCode::ArrowUp, KeyCode::KeyW])
}

pub fn is_down_just_pressed(input_state: &InputState) -> bool {
    any_keys_just_pressed(input_state, &[KeyCode::ArrowDown, KeyCode::KeyS])
}

pub fn is_back_just_pressed(input_state: &InputState) -> bool {
    any_keys_just_pressed(
        input_state,
        &[KeyCode::Escape, KeyCode::Backspace, KeyCode::Delete],
    )
}

pub fn is_select_just_pressed(input_state: &InputState) -> bool {
    input_state.keys[KeyCode::Enter].just_pressed()
        || input_state.keys[KeyCode::Space].just_pressed()
        || input_state.mouse.buttons[MouseButton::Left].just_pressed()
}
