
use gdnative::GodotObject;

pub struct Player<T: GodotObject> {
    pub ships: Vec<T>
}