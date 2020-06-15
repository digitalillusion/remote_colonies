use gdnative::GodotObject;

use std::cell::*;

use super::model::CelestialProperties;

pub enum PlayerAction {
    AddShip,
    MoveShips(CelestialProperties, CelestialProperties),
    None
}

pub struct Player<T: GodotObject> {
    pub planets: Vec<T>,
    pub ships: RefCell<Vec<T>>,
}


impl<T: GodotObject> Player<T> {

    pub fn new(planet: T, ship: T) -> Player<T> {
        Player {
            planets: vec!(planet),
            ships: RefCell::new(vec!(ship)),
        }
    }

    pub fn add_ship(&self, ship: T) {
        self.ships.borrow_mut().push(ship);
    }
}