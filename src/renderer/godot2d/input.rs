use gdnative::prelude::*;
use gdnative_bindings::*;

use std::time::SystemTime;

use crate::local::input::InputHandler;
use crate::local::model::CelestialProperties;
use crate::local::player::PlayerAction;

pub struct InputHandler2D {
    target_planet: Option<CelestialProperties>,
    primary_mouse_button_time: SystemTime,
}

impl Default for InputHandler2D {
    fn default() -> Self {
        InputHandler2D {
            target_planet: None,
            primary_mouse_button_time: SystemTime::now(),
        }
    }
}

impl InputHandler<CelestialProperties> for InputHandler2D {
    fn convert(&mut self, target: CelestialProperties, event: Ref<InputEvent>) -> PlayerAction {
        let input_event_mouse_button: Option<Ref<InputEventMouseButton>> = event.cast();
        if let Some(event) = input_event_mouse_button {
            return self.handle_mouse_button_event(target, event);
        }

        PlayerAction::Wait
    }
}

impl InputHandler2D {
    pub fn new() -> Self {
        InputHandler2D::default()
    }

    fn handle_mouse_button_event(
        &mut self,
        target: CelestialProperties,
        event: Ref<InputEventMouseButton>,
    ) -> PlayerAction {
        let mut player_action = PlayerAction::Wait;
        let event = unsafe { event.assume_safe() };
        if event.button_index() == 1 {
            if event.is_pressed() {
                self.primary_mouse_button_time = SystemTime::now();
            } else {
                let duration = SystemTime::now()
                    .duration_since(self.primary_mouse_button_time)
                    .unwrap();
                if duration.as_millis() < 500 {
                    player_action = PlayerAction::AddShip(target);
                } else if self.target_planet.is_some() {
                    let current = self.target_planet.unwrap();
                    if current.id != target.id {
                        player_action = PlayerAction::MoveShips(current, target);
                    }
                }
            }
        };
        self.target_planet.replace(target);
        player_action
    }
}
