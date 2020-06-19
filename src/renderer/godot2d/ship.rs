use gdnative::*;

use rand::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::f64::consts::FRAC_PI_2;


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

    #[export]
    unsafe fn reparent(&self, owner: RigidBody2D, planet_node: Area2D) {
        let mut planet_orbiters: Node2D = planet_node
            .find_node(GodotString::from_str("Orbiters"), false, true)
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        if let Some(mut parent) = owner.get_parent() {
            parent.remove_child(Some(owner.to_node()));
        }
        planet_orbiters.add_child(Some(owner.to_node()), false);
    }

    pub unsafe fn orbit(&self, mut owner: RigidBody2D, planet_node: Area2D, radius: f32) {
        let mut rng = rand::thread_rng();
        let angle = Angle::radians(rng.gen_range(0.0, 360.0));
        let position = Vector2::new(radius + 5.0,0.0).rotated(angle);
        owner.set_rotation(3.0 * FRAC_PI_2 + angle.radians as f64);
        owner.set_position(position);
        owner.call_deferred(GodotString::from("reparent"), &[Variant::from(planet_node)]);
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