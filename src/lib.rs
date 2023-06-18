//! # `remote_colonies` modules
//!
//! * `local` is the module that contains the game local logic
//! * `renderer` the graphical visualization of the game state
pub mod local;
pub mod renderer;

use gdnative::init::*;

use crate::renderer::godot2d::hud::HUD;
use crate::renderer::godot2d::planet::Planet;
use crate::renderer::godot2d::ship::Ship;
use crate::renderer::godot2d::Main;

fn init(handle: InitHandle) {
    handle.add_class::<Planet>();
    handle.add_class::<Ship>();
    handle.add_class::<HUD>();
    handle.add_class::<Main>();
}

godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();
