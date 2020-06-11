use gdnative::*;

use std::cell::RefCell;

use super::planet::Planet;

pub struct ShipProperties {
    pub owner: RigidBody2D
}

#[derive(NativeClass)]
#[inherit(RigidBody2D)]
pub struct Ship {
    properties: RefCell<ShipProperties>
}

#[methods]
impl Ship {
    
    fn _init(owner: RigidBody2D) -> Ship {
        let ship_properties = ShipProperties {
            owner
        };
        Ship {
            properties: RefCell::new(ship_properties)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: RigidBody2D) {
    }

    pub unsafe fn orbit(&self, planet: &Planet) {
        let mut props = self.properties.borrow_mut();
        let planet_props = planet.properties().borrow();
        props.owner.set_position(Vector2::new(planet_props.radius + 15.0, 0.0));
    }

    pub fn properties(&self) -> &RefCell<ShipProperties> {
        &self.properties
    }

    pub unsafe fn with_mut<F>(node: RigidBody2D, mut with_fn: F)
    where
        F: FnMut(&mut Ship) -> ()
    {
        let instance = Instance::<Ship>::try_from_base(node).unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap();
    }

    pub unsafe fn with<F>(node: RigidBody2D, with_fn: F)
    where
        F: Fn(&Ship) -> ()
    {
        let instance = Instance::<Ship>::try_from_base(node).unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap();
    }
    
}