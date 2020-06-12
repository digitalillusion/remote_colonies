use gdnative::*;

use std::cell::RefCell;

use crate::local::model::Vessel;

pub struct ShipProperties {
    pub owner: Area2D
}

#[derive(NativeClass)]
#[inherit(Area2D)]
pub struct Ship {
    properties: RefCell<ShipProperties>
}

impl Vessel for Ship {
    
}

#[methods]
impl Ship {
    
    fn _init(owner: Area2D) -> Ship {
        let ship_properties = ShipProperties {
            owner
        };
        Ship {
            properties: RefCell::new(ship_properties)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: Area2D) {
    }

    pub fn properties(&self) -> &RefCell<ShipProperties> {
        &self.properties
    }

    pub unsafe fn with_mut<F, T>(node: Area2D, mut with_fn: F) -> T
    where
        F: FnMut(&mut Ship) -> T
    {
        let instance = Instance::<Ship>::try_from_base(node).unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub unsafe fn with<F, T>(node: Area2D, with_fn: F) -> T
    where
        F: Fn(&Ship) -> T
    {
        let instance = Instance::<Ship>::try_from_base(node).unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }
    
}