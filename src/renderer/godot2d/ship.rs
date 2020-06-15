use gdnative::*;

use std::cell::RefCell;

use crate::local::model::*;



#[derive(NativeClass)]
#[inherit(Area2D)]
pub struct Ship {
    // owner: RefCell<Area2D>,
    properties: RefCell<VesselProperties>
}

impl Vessel for Ship {
    fn properties(&self) -> &RefCell<VesselProperties> {
        &self.properties
    }
}

#[methods]
impl Ship {
    
    fn _init(_owner: Area2D) -> Ship {
        let properties = VesselProperties {
        };
        Ship {
            // owner: RefCell::new(owner),
            properties: RefCell::new(properties)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: Area2D) {
    }

    pub fn properties(&self) -> &RefCell<VesselProperties> {
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