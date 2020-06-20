use gdnative::{Color, GodotObject};

use std::cell::*;

use super::model::*;

pub enum PlayerAction {
    AddShip,
    MoveShips(CelestialProperties, CelestialProperties),
    None
}

pub struct Player<T: GodotObject, U: GodotObject> {
    pub planets:RefCell<Vec<T>>,
    pub ships: RefCell<Vec<U>>,
    properties: RefCell<ContenderProperties>
}

impl<T: GodotObject, U: GodotObject> Contender for Player<T, U> {
    fn properties(&self) -> &RefCell<ContenderProperties> {
        &self.properties
    }
}


impl<T: GodotObject, U: GodotObject> Player<T, U> {

    pub fn new(id: usize, planet: T, ship: U) -> Player<T, U> {
        
        let properties = ContenderProperties {
            id,
            color: get_color(id)
        };
        Player {
            properties: RefCell::new(properties),
            planets: RefCell::new(vec!(planet)),
            ships: RefCell::new(vec!(ship)),
        }
    }

    pub fn add_ship(&self, ship: U) {
        self.ships.borrow_mut().push(ship);
    }
}

fn get_color(id: usize) -> Color {
    let colors = [
        (230, 25, 75), // Red
        (60, 180, 75), // Green
        (255, 225, 25), // Yellow
        (0, 130, 200), // Blue
        (245, 130, 48), // Orange
        (145, 30, 180), // Purple
        (70, 240, 240), // Cyan
        (240, 50, 230), // Magenta
        (210, 245, 60), // Lime
        (250, 190, 212), // Pink
        (0, 128, 128), // Teal
        (220, 190, 255), // Lavender
        (170, 110, 40), // Brown
        (255, 250, 200), // Beige
        (128, 0, 0), // Maroon
        (170, 255, 195), // Mint
        (128, 128, 0), // Olive
        (255, 215, 180), // Apricot
        (0, 0, 128), // Navy
        (128, 128, 128), // Grey
    ];
    Color::rgb(
        colors[id].0 as f32 / 255.0, 
        colors[id].1 as f32 / 255.0, 
        colors[id].2 as f32 / 255.0
    )
}