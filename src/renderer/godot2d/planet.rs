use gdnative::prelude::*;
use gdnative_bindings::*;

use rand::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::local::planet::PlanetBusiness;
use crate::local::player::*;
use crate::renderer::godot2d::ship::{RefShipNode2D, Ship};

use super::input::InputHandler2D;
use super::instance_scene;
use super::player::Player2D;
use super::starmap::Starmap2D;
use crate::local::input::InputHandler;
use crate::local::model::*;
use crate::local::GameState;

pub type RefPlanetNode2D = Ref<Node2D>;

type PlanetPlayerAction = dyn Fn(Box<&Planet>, PlayerAction);

#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct Planet {
    #[property]
    ship: Ref<PackedScene>,

    game_state: Option<Rc<RefCell<GameState<Starmap2D, Player2D>>>>,
    business: PlanetBusiness,
    owner: RefPlanetNode2D,
    properties: RefCell<CelestialProperties>,
    input_handler_fn: Option<Box<PlanetPlayerAction>>,
    input_handler: Option<Rc<RefCell<InputHandler2D>>>,
}

impl Celestial for Planet {
    fn properties(&self) -> CelestialProperties {
        *self.properties.borrow()
    }
}

#[methods]
impl Planet {
    fn new(owner: &Node2D) -> Self {
        let mut rng = rand::thread_rng();
        let owner = unsafe { owner.assume_unique() }.cast::<Node2D>().unwrap();
        let resources_initial =
            rng.gen_range((Consts::PLANET_RESOURCES_INIT * 0.1)..Consts::PLANET_RESOURCES_INIT);

        let properties = CelestialProperties {
            id: 0,
            contender_id: usize::MAX,
            radius: 0.0,
            resources: resources_initial,
            resources_increase: resources_initial
                * rng.gen_range((Consts::PLANET_RESOURCES_INC * 0.1)..Consts::PLANET_RESOURCES_INC),
            extracted: 0.0,
        };
        Planet {
            ship: PackedScene::new().into_shared(),
            owner: owner.into_shared(),
            properties: RefCell::new(properties),
            input_handler_fn: None,
            input_handler: None,
            game_state: None,
            business: PlanetBusiness::new(),
        }
    }

    #[method]
    pub fn _ready(&self, #[base] _owner: &Node2D) {}

    #[method]
    pub fn _on_planet_gui_input(
        &self,
        #[base] _owner: &Node2D,
        _viewport: Ref<Node>,
        event: Ref<InputEvent>,
        _shape_idx: isize,
    ) {
        let target = Box::new(self);
        let player_action = self
            .input_handler
            .as_ref()
            .unwrap()
            .borrow_mut()
            .convert(self.properties(), event);
        self.input_handler_fn.as_ref().unwrap()(target, player_action);
    }

    #[method]
    pub fn _on_ship_arrival(&self, #[base] owner: &Node2D, ship_node: RefShipNode2D) {
        let props = self.properties();
        let ship_node_obj: &RigidBody2D = unsafe { ship_node.assume_safe() }.as_ref();
        if ship_node_obj.linear_velocity().length() == 0.0
            || ship_node_obj.get_angle_to(owner.global_position()).abs() > 0.004
        {
            return;
        }
        ship_node_obj.set_linear_velocity(Vector2::new(0.0, 0.0));

        Ship::with_mut(&ship_node, |ship| {
            ship.orbit(ship_node_obj, props.id, self.owner, props.radius);
        });
    }

    #[method]
    pub fn _on_resource_timer_timeout(&self, #[base] owner: &Node2D) {
        let mut props = self.properties.borrow_mut();
        let planet_orbiters = unsafe {
            owner
                .get_node_as::<Node2D>("Orbiters")
                .expect("Cannot resolve Orbiters")
        };
        let mut ships_count = 0;
        for index in 0..planet_orbiters.get_child_count() {
            let orbiter =
                unsafe { planet_orbiters.get_child(index).unwrap().assume_safe() }.as_ref();
            let orbiter: &RigidBody2D = orbiter.cast().unwrap();
            let orbiter = unsafe { orbiter.assume_shared() };
            Ship::with_mut(&orbiter, |ship| {
                if ship.properties().contender_id == props.contender_id {
                    ships_count += 1;
                }
            })
        }
        self.business.resources_update(&mut props, ships_count);

        let label = &format!("{}/{}", props.resources as usize, props.extracted as usize);
        let planet_label = unsafe {
            owner
                .get_node_as::<Label>("Label")
                .expect("Cannot resolve Label")
        };
        planet_label.set_text(label);
    }

    #[method]
    pub fn _on_orbiters_timer_timeout(&self, #[base] owner: &Node2D) {
        let planet_orbiters = unsafe {
            owner
                .get_node_as::<Node2D>("Orbiters")
                .expect("Cannot resolve Orbiters")
                .assume_unique()
        };
        let rotation = planet_orbiters.global_rotation();
        planet_orbiters.set_global_rotation(rotation - 0.001 * Consts::MOVE_SHIP_SPEED_MULT as f64);
    }

    #[method]
    pub fn _process(&self, #[base] owner: &Node2D, _delta: f64) {
        let game_state = self.get_game_state();
        let players = game_state.get_players();

        let ships_by_player_on_planet = game_state.get_ships_by_player_on_planet(self.properties());
        let (winner, casualties) = self.business.battle(ships_by_player_on_planet);

        for casualty in casualties {
            if let Some(casualty_player) = players
                .iter()
                .find(|player| player.properties().id == casualty.contender_id)
            {
                let mut casualty_player_ships = casualty_player.ships.borrow_mut();
                let (index, casualty) = casualty_player_ships
                    .iter_mut()
                    .enumerate()
                    .find(|(_, ship_node)| {
                        Ship::with(ship_node, |ship| {
                            ship.properties().id == casualty.id
                                && ship.properties().celestial_id == self.properties().id
                        })
                    })
                    .unwrap();
                unsafe { casualty.assume_safe() }.queue_free();
                casualty_player_ships.remove(index);
            };
        }

        if let Some(winner) = winner {
            let winner = players
                .iter()
                .find(|player| player.properties().id == winner.id)
                .unwrap();
            let winner_props = winner.properties();
            let winner_planet = unsafe { owner.assume_shared() };
            if let Some(loser) = players
                .iter()
                .find(|player| player.properties().id == self.properties().contender_id)
            {
                if loser.properties().id != winner_props.id {
                    loser.planets.borrow_mut().retain(|planet| {
                        Planet::with(planet, |planet| {
                            planet.properties().id != self.properties().id
                        })
                    });
                    self.properties.borrow_mut().contender_id = winner_props.id;
                    winner.planets.borrow_mut().push(winner_planet);
                }
            } else {
                self.properties.borrow_mut().contender_id = winner_props.id;
                winner.planets.borrow_mut().push(winner_planet);
            }
            let planet_sprite = unsafe {
                self.owner
                    .assume_safe()
                    .get_node_as::<Sprite>("Area2D/Sprite")
            }
            .expect("Cannot resolve Area2D/Sprite");
            planet_sprite.set_modulate(winner_props.color);
        }
    }

    pub fn add_ship(&self, resources_cost: f32, player: &Player2D) {
        let mut props = self.properties.borrow_mut();

        if self
            .business
            .can_add_ship(&mut props, player.properties(), resources_cost)
        {
            let ship_node: Ref<RigidBody2D, _> = instance_scene(&self.ship);
            let ship_node = ship_node.into_shared();
            let ship_node_obj: &RigidBody2D = unsafe { ship_node.assume_safe() }.as_ref();
            player.add_ship(ship_node);
            let ships_count = player.ships.borrow().len();

            Ship::with_mut(&ship_node, |ship| {
                ship.set_id(player.properties(), ships_count);
                ship.orbit(ship_node_obj, props.id, self.owner, props.radius);
            });
        }
    }

    pub fn add_player(&self, is_bot: bool) {
        let mut props = self.properties.borrow_mut();
        let ship_node: Ref<RigidBody2D, _> = instance_scene(&self.ship);
        let ship_node = ship_node.into_shared();
        let ship_node_obj: &RigidBody2D = unsafe { ship_node.assume_safe() }.as_ref();

        let mut game_state = self.game_state.as_ref().unwrap().borrow_mut();
        props.contender_id = game_state.get_players().len();
        let player = Player2D::new(props.contender_id, self.owner, ship_node, is_bot);
        let ships_count = player.ships.borrow().len();
        let planet_sprite = unsafe {
            self.owner
                .assume_safe()
                .get_node_as::<Sprite>("Area2D/Sprite")
                .expect("Cannot resolve Area2D/Sprite")
        };
        planet_sprite.set_modulate(player.properties().color);

        Ship::with_mut(&ship_node, |ship| {
            ship.set_id(player.properties(), ships_count);
            ship.orbit(ship_node_obj, props.id, self.owner, props.radius);
        });

        game_state.add_player(Rc::new(player));
    }

    pub fn move_ships(&self, percent: usize, player: &Player2D, destination: &RefPlanetNode2D) {
        let planet_orbiters = unsafe {
            self.owner
                .assume_safe()
                .get_node_as::<Node2D>("Orbiters")
                .expect("Cannot resolve Orbiters")
        };
        let mut selected_ships: Vec<RefShipNode2D> = Vec::new();
        for index in 0..planet_orbiters.get_child_count() {
            let orbiter =
                unsafe { planet_orbiters.get_child(index).unwrap().assume_safe() }.as_ref();
            let orbiter: &RigidBody2D = orbiter.cast().unwrap();
            let orbiter = unsafe { orbiter.assume_shared() };
            let is_player_ship = Ship::with(&orbiter, |ship| {
                ship.properties().contender_id == player.properties().id
            });
            if is_player_ship {
                selected_ships.push(orbiter)
            }
        }
        let count: usize = self
            .business
            .count_ships_to_move(selected_ships.len(), percent);
        let selected_ships = selected_ships.drain(0..count);
        let root_node = unsafe { self.owner.assume_safe() }
            .as_ref()
            .get_parent()
            .unwrap();

        for ship_node in selected_ships {
            let ship_node_obj: &RigidBody2D =
                unsafe { ship_node.assume_safe() }.as_ref().cast().unwrap();
            Ship::with(&ship_node, |ship| ship.leave_orbit());

            let position = ship_node_obj.global_position();
            let parent_ref = unsafe { ship_node_obj.get_parent().unwrap().assume_safe() }.as_ref();
            let ship_instance: TInstance<Ship> =
                unsafe { ship_node.assume_safe() }.cast_instance().unwrap();
            parent_ref.remove_child(ship_instance.clone());

            unsafe { root_node.assume_safe() }
                .as_ref()
                .add_child(ship_instance, false);
            ship_node_obj.set_global_position(position);

            let destination_obj = unsafe { destination.assume_safe() }.as_ref();
            ship_node_obj.look_at(destination_obj.global_position());
            ship_node_obj.set_linear_velocity(
                (destination_obj.global_position() - position).normalized()
                    * 10.0
                    * Consts::MOVE_SHIP_SPEED_MULT,
            );
        }
    }

    pub fn set_random_features(&self) {
        let mut props = self.properties.borrow_mut();
        let owner = unsafe { self.owner.assume_safe() }.as_ref();

        let viewport_rect: Rect2 = owner.get_viewport_rect();
        let viewport_width = viewport_rect.size.x;
        let viewport_height = viewport_rect.size.y;

        let mut rng = rand::thread_rng();
        let planet_area = unsafe {
            owner
                .get_node_as::<Area2D>("Area2D")
                .expect("Cannot resolve Area2D")
        };
        let planet_sprite = unsafe {
            owner
                .get_node_as::<Sprite>("Area2D/Sprite")
                .expect("Cannot resolve Area2D/Sprite")
        };
        let size = unsafe {
            planet_sprite
                .texture()
                .expect("Unable to get Texture")
                .assume_safe()
        }
        .get_size()
        .x * 0.5;
        let scale = rng.gen_range(0.5..2.5) * planet_sprite.scale().x;
        let scale_vector = Vector2::new(scale, scale);
        planet_area.set_scale(scale_vector);

        props.radius = 0.42 * scale * size;
        let diameter = 2.0 * props.radius;
        let x_offset =
            (rng.gen_range(0.0..1.0) * viewport_width).clamp(diameter, viewport_width - diameter);
        let y_offset =
            (rng.gen_range(0.0..1.0) * viewport_height).clamp(diameter, viewport_height - diameter);
        owner.set_position(Vector2::new(x_offset, y_offset));
    }

    pub fn set_id(&self, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.id = id;
    }

    pub fn get_game_state(&self) -> std::cell::Ref<GameState<Starmap2D, Player2D>> {
        self.game_state.as_ref().unwrap().borrow()
    }

    pub fn set_input_handler<F: 'static>(
        &mut self,
        input_handler: Rc<RefCell<InputHandler2D>>,
        input_handler_fn: F,
    ) where
        F: Fn(Box<&Planet>, PlayerAction),
    {
        self.input_handler = Some(input_handler);
        self.input_handler_fn = Some(Box::new(input_handler_fn));
    }

    pub fn set_game_state(&mut self, game_state: Rc<RefCell<GameState<Starmap2D, Player2D>>>) {
        self.game_state = Some(game_state);
    }

    pub fn set_resources(&self, initial: f32, inc: f32) {
        let mut props = self.properties.borrow_mut();
        self.business.resources_init(&mut props, initial, inc);
    }

    pub fn with_mut<F, T>(base: &RefPlanetNode2D, mut with_fn: F) -> T
    where
        F: FnMut(&mut Planet) -> T,
    {
        let instance = unsafe { base.assume_safe() }.cast_instance().unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub fn with<F, T>(base: &RefPlanetNode2D, with_fn: F) -> T
    where
        F: Fn(&Planet) -> T,
    {
        let instance = unsafe { base.assume_safe() }.cast_instance().unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }

    pub fn get_by_id(planets: &[RefPlanetNode2D], id: usize) -> &RefPlanetNode2D {
        planets
            .iter()
            .find(|p| Planet::with(p, |planet| planet.properties().id == id))
            .unwrap()
    }
}
