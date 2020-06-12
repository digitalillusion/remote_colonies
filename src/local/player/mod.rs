use gdnative::GodotObject;

pub struct Player<T: GodotObject> {
    pub planets: Vec<T>,
    pub ships: Vec<T>,
}

impl<T: GodotObject> Player<T> {

    pub fn new(planet: T, ship: T) -> Player<T> {
        Player {
            planets: vec!(planet),
            ships: vec!(ship),
        }
    }
}