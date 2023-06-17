use gdnative::prelude::*;
use gdnative_bindings::*;

use rand::*;

use std::cell::RefCell;
use std::f64::consts::FRAC_PI_2;
use std::rc::Rc;

use super::player::Player2D;
use crate::local::model::*;
use crate::renderer::godot2d::planet::RefPlanetNode2D;

pub type RefShipNode2D = Ref<RigidBody2D>;

#[derive(NativeClass)]
#[inherit(RigidBody2D)]
pub struct Ship {
    owner: RefShipNode2D,
    properties: RefCell<VesselProperties>,
}

impl Vessel for Ship {
    fn properties(&self) -> VesselProperties {
        *self.properties.borrow()
    }
}

#[methods]
impl Ship {
    fn new(owner: &RigidBody2D) -> Ship {
        let owner = unsafe { owner.assume_unique() }
            .cast::<RigidBody2D>()
            .unwrap();
        let properties = VesselProperties {
            id: 0,
            contender_id: 0,
            celestial_id: 0,
        };
        Ship {
            owner: owner.into_shared(),
            properties: RefCell::new(properties),
        }
    }

    #[method]
    fn _ready(&self, #[base] _owner: &RigidBody2D) {}

    #[method]
    fn reparent(&self, #[base] owner: &RigidBody2D, planet_node: Ref<Node2D>) {
        let planet_orbiters = unsafe {
            planet_node
                .assume_safe()
                .get_node_as::<Node2D>("Orbiters")
                .expect("Cannot resolve Orbiters")
        };
        let ship_instance: TInstance<Ship> = unsafe { owner.assume_shared().assume_safe() }
            .cast_instance()
            .unwrap();
        if let Some(parent) = owner.get_parent() {
            let parent_ref = unsafe { parent.assume_safe() }.as_ref();
            parent_ref.remove_child(ship_instance.clone());
        }
        planet_orbiters.add_child(ship_instance, false);
    }

    pub fn orbit(
        &self,
        owner: &RigidBody2D,
        celestial_id: usize,
        planet_node: RefPlanetNode2D,
        radius: f32,
    ) {
        let mut props = self.properties.borrow_mut();
        props.celestial_id = celestial_id;

        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..2.0);
        let position = Vector2::new(radius, 0.0).rotated(angle);
        owner.set_rotation(3.0 * FRAC_PI_2 + angle as f64);
        owner.set_position(position);
        unsafe { owner.call_deferred("reparent", &[planet_node.to_variant()]) };
    }

    pub fn find_player(&self, players: &[Rc<Player2D>]) -> Option<Rc<Player2D>> {
        let props = self.properties();
        let player = players
            .iter()
            .find(|p| {
                let player_props = p.properties();
                player_props.id == props.contender_id
            })
            .unwrap();

        if player
            .ships
            .borrow()
            .iter()
            .any(|s| Ship::with(s, |ship| ship.properties().id == props.id))
        {
            return Some(player.clone());
        }
        None
    }

    pub fn set_id(&self, player_props: ContenderProperties, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.contender_id = player_props.id;
        props.id = id;

        let ship_sprite = unsafe {
            self.owner
                .assume_safe()
                .get_node_as::<Sprite>("Sprite")
                .expect("Cannot resolve Sprite")
        };
        ship_sprite.set_modulate(player_props.color);
    }

    pub fn leave_orbit(&self) {
        let mut props = self.properties.borrow_mut();
        props.celestial_id = usize::MAX;
    }

    pub fn with_mut<F, T>(base: &RefShipNode2D, mut with_fn: F) -> T
    where
        F: FnMut(&mut Ship) -> T,
    {
        let instance = unsafe { base.assume_safe() }.cast_instance().unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub fn with<F, T>(base: &RefShipNode2D, with_fn: F) -> T
    where
        F: Fn(&Ship) -> T,
    {
        let instance = unsafe { base.assume_safe() }.cast_instance().unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }
}
