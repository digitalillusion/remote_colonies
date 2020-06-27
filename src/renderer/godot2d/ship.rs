use gdnative::*;

use rand::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::f64::consts::FRAC_PI_2;

use super::player::Player2D;
use crate::local::model::*;



#[derive(NativeClass)]
#[inherit(RigidBody2D)]
pub struct Ship {
    owner: RigidBody2D,
    properties: RefCell<VesselProperties>
}

impl Vessel for Ship {
    fn properties(&self) -> VesselProperties {
        *self.properties.borrow()
    }
}

#[methods]
impl Ship {
    
    fn _init(owner: RigidBody2D) -> Ship {
        let properties = VesselProperties {
            id: 0,
            contender_id: 0,
            celestial_id: 0,
        };
        Ship {
            owner: owner,
            properties: RefCell::new(properties)
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: RigidBody2D) {
    }

    #[export]
    unsafe fn reparent(&self, owner: RigidBody2D, planet_node: Node2D) {
        let mut planet_orbiters: Node2D = planet_node
            .get_node(NodePath::from_str("Orbiters"))
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        if let Some(mut parent) = owner.get_parent() {
            parent.remove_child(Some(owner.to_node()));
        }
        planet_orbiters.add_child(Some(owner.to_node()), false);
    }

    pub unsafe fn orbit(&self, mut owner: RigidBody2D, celestial_id: usize, planet_node: Node2D, radius: f32) {
        let mut props = self.properties.borrow_mut();
        props.celestial_id = celestial_id;

        let mut rng = rand::thread_rng();
        let angle = Angle::radians(rng.gen_range(0.0, 360.0));
        let position = Vector2::new(radius,0.0).rotated(angle);
        owner.set_rotation(3.0 * FRAC_PI_2 + angle.radians as f64);
        owner.set_position(position);
        owner.call_deferred(GodotString::from("reparent"), &[Variant::from(planet_node)]);
    }

    pub unsafe fn find_player(&self, players: &Vec<Rc<Player2D>>) -> Option<Rc<Player2D>> {       
        let props = self.properties();
        let player = players.iter()
            .find(|p| {
                let player_props = p.properties();
                player_props.id == props.contender_id
            })
            .unwrap();

        if player.ships.borrow().iter()
            .find(|s| {
                Ship::with(**s, |ship| {
                    ship.properties().id == props.id    
                })
            }).is_some() {
            return Some(player.clone())
        }
        None
    }

    pub unsafe fn set_id(&self, player_props: ContenderProperties, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.contender_id = player_props.id;
        props.id = id;

        let mut ship_sprite: Sprite = self.owner
            .get_node(NodePath::from_str("Sprite"))
            .expect("Unable to find ship/Shape")
            .cast()
            .expect("Unable to cast to Sprite");
        ship_sprite.set_modulate(player_props.color);
    }

    pub fn leave_orbit(&self) {
        let mut props = self.properties.borrow_mut();
        props.celestial_id = usize::MAX;
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