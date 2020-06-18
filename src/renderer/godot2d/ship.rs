use gdnative::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::local::player::*;
use crate::local::model::*;



#[derive(NativeClass)]
#[inherit(RigidBody2D)]
pub struct Ship {
    properties: RefCell<VesselProperties>
}

impl Vessel for Ship {
    fn properties(&self) -> &RefCell<VesselProperties> {
        &self.properties
    }
}

#[methods]
impl Ship {
    
    fn _init(_owner: RigidBody2D) -> Ship {
        let properties = VesselProperties {
            id: 0,
            player_id: 0,
        };
        Ship {
            properties: RefCell::new(properties)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: RigidBody2D) {
    }

    pub fn properties(&self) -> &RefCell<VesselProperties> {
        &self.properties
    }

    pub unsafe fn find_player(&self, players: &Vec<Rc<Player<Area2D, RigidBody2D>>>) -> Option<Rc<Player<Area2D, RigidBody2D>>> {       
        let props = self.properties.borrow();
        let player = players.iter()
            .find(|p| p.id == props.player_id)
            .unwrap();

        if player.ships.borrow().iter()
            .find(|s| {
                Ship::with(**s, |ship| {
                    ship.properties().borrow().id == props.id    
                })
            }).is_some() {
            return Some(player.clone())
        }
        None
    }

    pub fn set_id(&self, player_id: usize, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.player_id = player_id;
        props.id = id;
    }

    pub unsafe fn with_mut<F, T>(node: RigidBody2D, mut with_fn: F) -> T
    where
        F: FnMut(&mut Ship) -> T
    {
        let instance = Instance::<Ship>::try_from_base(node).unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub unsafe fn with<F, T>(node: RigidBody2D, with_fn: F) -> T
    where
        F: Fn(&Ship) -> T
    {
        let instance = Instance::<Ship>::try_from_base(node).unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }
    
}