
use gdnative::GodotObject;

pub struct Player<T: GodotObject, S: GodotObject> {
    planets: Vec<T>,
    ships: Vec<S>,
}

impl<T: GodotObject, S: GodotObject> Player<T, S> {

    pub fn new(planet: T, ship: S) -> Player<T, S> {
        Player {
            planets: vec!(planet),
            ships: vec!(ship),
        }
    }

    pub fn get_ships(&self) -> &Vec<S> {
        &self.ships
    }

    pub fn get_planets(&self) -> &Vec<T> {
        &self.planets
    }
}