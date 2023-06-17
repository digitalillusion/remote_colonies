use gdnative::prelude::*;

use super::player::PlayerAction;

pub trait InputHandler<T> {
    fn convert(&mut self, target: T, event: Ref<InputEvent>) -> PlayerAction;
}
