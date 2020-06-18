use gdnative::GodotObject;

use std::cell::*;

use super::model::CelestialProperties;

pub enum PlayerAction {
    AddShip,
    MoveShips(CelestialProperties, CelestialProperties),
    None
}

pub struct Player<T: GodotObject, U: GodotObject> {
    pub id: usize,
    pub planets:RefCell<Vec<T>>,
    pub ships: RefCell<Vec<U>>,
}


impl<T: GodotObject, U: GodotObject> Player<T, U> {

    pub fn new(id: usize, planet: T, ship: U) -> Player<T, U> {
        Player {
            id,
            planets: RefCell::new(vec!(planet)),
            ships: RefCell::new(vec!(ship)),
        }
    }

    pub fn add_ship(&self, ship: U) {
        self.ships.borrow_mut().push(ship);
    }
}