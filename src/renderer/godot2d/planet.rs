use gdnative::*;

use rand::*;

use std::cell::*;
use std::rc::Rc;

use crate::local::player::*;
use crate::local::planet::PlanetBusiness;
use crate::renderer::godot2d::ship::Ship;

use crate::local::model::*;
use crate::local::MainLoop;
use crate::local::input::InputHandler;
use super::instance_scene;
use super::input::InputHandler2D;
use super::player::Player2D;
use super::starmap::Starmap2D;

#[derive(NativeClass)]
#[inherit(Node2D)]

pub struct Planet {

    #[property]
    ship: PackedScene,

    main_loop: Option<Rc<RefCell<MainLoop<Starmap2D, Player2D>>>>,
    business: PlanetBusiness,
    owner: Node2D,
    properties: RefCell<CelestialProperties>,
    input_handler_fn: Option<Box<dyn Fn(Box<&Planet>, PlayerAction) -> ()>>,
    input_handler: Option<Rc<RefCell<InputHandler2D>>>,
}

impl Celestial for Planet {
    fn properties(&self) -> CelestialProperties {
        *self.properties.borrow()
    }
}

#[methods]
impl Planet {
    
    fn _init(owner: Node2D) -> Planet {
        let mut rng = rand::thread_rng();
        let resources_initial  = rng.gen_range(10.0, 250.0);

        let properties = CelestialProperties {
            id: 0,
            contender_id: 0,
            radius: 0.0,
            resources: resources_initial,
            resources_increase: resources_initial * rng.gen_range(0.0002, 0.005),
            extracted: 0.0,
        };
        Planet {
            ship: PackedScene::new(),
            owner,
            properties: RefCell::new(properties),
            input_handler_fn: None,
            input_handler: None,
            main_loop: None,
            business: PlanetBusiness::new()
        }
    }
    
    #[export]
    unsafe fn _ready(&self, _owner: Node2D) {
    }

    #[export]
    pub unsafe fn _on_planet_gui_input(&self, _owner: Node2D, _viewport: Node, event: InputEvent, _shape_idx: isize) {
        let target = Box::new(self);
        let player_action = self.input_handler.as_ref().unwrap().borrow_mut()
            .convert(self.properties(), event);
        self.input_handler_fn.as_ref().unwrap()(target, player_action);
    }

    #[export]
    pub unsafe fn _on_ship_arrival(&self, owner: Node2D, ship_node: Node) {
        let props = self.properties();
        let mut ship_node: RigidBody2D = ship_node.cast().unwrap();
        if ship_node.get_linear_velocity().length() == 0.0 || ship_node.get_angle_to(owner.get_global_position()).abs() > 0.005 {
            return;
        }
        ship_node.set_linear_velocity(Vector2::new(0.0, 0.0));
        
        Ship::with_mut(ship_node, |ship| {
            ship.orbit(ship_node, props.id, owner, props.radius);
        });
    }

    #[export]
    pub unsafe fn _on_resource_timer_timeout(&self, owner: Node2D) {
        let mut props = self.properties.borrow_mut();
        let planet_orbiters: Node2D = owner
            .get_node(NodePath::from_str("Orbiters"))
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        let orbiters_count = planet_orbiters.get_children().len();
        self.business.resources_update(&mut props, orbiters_count);

        let label = &format!("{} - {}/{}", 
            props.id.to_string(), 
            (props.resources as usize).to_string(),
            (props.extracted as usize).to_string()
        );
        let mut planet_label: Label = owner
            .get_node(NodePath::from_str("Label"))
            .expect("Unable to find planet/Label")
            .cast()
            .expect("Unable to cast to Label");
        planet_label.set_text(GodotString::from_str(label));
    }

    #[export]
    pub unsafe fn _on_orbiters_timer_timeout(&self, owner: Node2D) {
        let mut planet_orbiters: Node2D = owner
            .get_node(NodePath::from_str("Orbiters"))
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        let rotation = planet_orbiters.get_global_rotation();
        planet_orbiters.set_global_rotation(rotation - 0.01);
    }

    #[export]
    pub unsafe fn _process(&self, _owner: Node2D, _delta: f64) {
        let owner = self.owner;
        let main_loop = self.get_main_loop();
        let players = &main_loop.players;

        let ships_by_player = main_loop.get_ships_by_player(self.properties());
        let (winner, casualties) = self.business.battle(ships_by_player);
        
        for casualty in casualties {
            if let Some(casualty_player) = players.iter().find(|player| {
                player.properties().id == casualty.contender_id
            }) {
                let mut casualty_player_ships = casualty_player.ships.borrow_mut();
                let (index, casualty) = casualty_player_ships.iter_mut()
                    .enumerate()
                    .find(|(_, ship_node)| {
                        Ship::with(**ship_node, |ship| {
                            ship.properties().id == casualty.id
                        })
                    }).unwrap();
                casualty.free();
                casualty_player_ships.remove(index);
            };
        }
        
        if let Some(winner) = winner {
            let winner = players.iter().find(|player| {
                player.properties().id == winner.id
            }).unwrap();
            let winner_props = winner.properties();
            if let Some(loser) = players.iter().find(|player| {
                player.properties().id == self.properties().contender_id
            }) {
                if loser.properties().id != winner_props.id {
                    loser.planets.borrow_mut().retain(|planet| {
                        Planet::with(*planet, |planet| {
                            planet.properties().id != self.properties().id    
                        })  
                    });
                    self.properties.borrow_mut().contender_id = winner_props.id;
                    winner.planets.borrow_mut().push(self.owner);    
                }            
            } else {
                winner.planets.borrow_mut().push(self.owner);
            }
            let mut planet_sprite: Sprite = owner
                .get_node(NodePath::from_str("Area2D/Sprite"))
                .expect("Unable to find planet/Area2D/Sprite")
                .cast()
                .expect("Unable to cast to Sprite");
            planet_sprite.set_modulate(winner_props.color);
        }
    }

    pub unsafe fn add_ship(&self, resources_cost: f32, player: &Player2D) {
        let mut props = self.properties.borrow_mut();
        let owner = self.owner;

        if self.business.can_add_ship(&mut props, player.properties(), resources_cost) {
            let ship_node: RigidBody2D = instance_scene(&self.ship).unwrap();
            player.add_ship(ship_node);
            let ships_count = player.ships.borrow().len();

            Ship::with_mut(ship_node, |ship| {
                ship.set_id(player.properties(), ships_count);
                ship.orbit(ship_node, props.id, owner, props.radius);
            });
        }
    }

    pub unsafe fn add_player(&self) {
        let mut props = self.properties.borrow_mut();
        let owner = self.owner;
        let ship_node: RigidBody2D = instance_scene(&self.ship).unwrap();
        
        let mut main_loop = self.main_loop.as_ref().unwrap().borrow_mut();
        props.contender_id = main_loop.players.len();
        let player: Player2D = Player::new(props.contender_id, owner, ship_node);
        let ships_count = player.ships.borrow().len();
        let mut planet_sprite: Sprite = owner
            .get_node(NodePath::from_str("Area2D/Sprite"))
            .expect("Unable to find planet/Area2D/Sprite")
            .cast()
            .expect("Unable to cast to Sprite");
        planet_sprite.set_modulate(player.properties().color);
        
        Ship::with_mut(ship_node, |ship| {
            ship.set_id(player.properties(), ships_count);
            ship.orbit(ship_node, props.id, owner, props.radius);
        });

        main_loop.players.push(Rc::new(player));
    }

    pub unsafe fn move_ships(&self, percent: usize, player: &Player2D, destination: &Rc<Node2D>) {
        let owner = self.owner;
        let planet_orbiters: Node2D = owner
            .get_node(NodePath::from_str("Orbiters"))
            .expect("Unable to find planet/Orbiters")
            .cast()
            .expect("Unable to cast to Node2D");
        let mut selected_ships: Vec<RigidBody2D> = planet_orbiters.get_children().iter().filter_map(|child| {
            let child: RigidBody2D = child.try_to_object().unwrap();
            let is_player_ship = Ship::with(child, |ship| {
                ship.properties().contender_id == player.properties().id
            });
            if is_player_ship  {
                return Some(child)
            }
            None
        })
        .collect();
        let count: usize =  self.business.count_ships_to_move(selected_ships.len(), percent);
        let selected_ships = selected_ships.drain(0..count);
        let mut root_node = self.owner.get_parent().unwrap();

        for mut ship in selected_ships {
            let position = ship.get_global_position();
            let ship_node = Some(ship.to_node());
            ship.get_parent().unwrap().remove_child(ship_node);
            root_node.add_child(ship_node, false);
            ship.set_global_position(position);
            
            ship.look_at(destination.get_global_position());
            ship.set_linear_velocity((destination.get_global_position() - position).normalize() * 10.0);
        }
    }

    pub unsafe fn set_random_features(&self) {
        let mut props = self.properties.borrow_mut();
        let mut owner = self.owner;

        let viewport_rect: Rect2 = owner.get_viewport_rect();
        let viewport_width = viewport_rect.width();
        let viewport_height = viewport_rect.height();

        let mut rng = rand::thread_rng();
        let mut planet_area: Area2D = owner
            .get_node(NodePath::from_str("Area2D"))
            .expect("Unable to find planet/Area2D")
            .cast()
            .expect("Unable to cast to Area2D");
        let planet_sprite: Sprite = planet_area
            .get_node(NodePath::from_str("Sprite"))
            .expect("Unable to find planet/Area2D/Sprite")
            .cast()
            .expect("Unable to cast to Sprite");
        let size = planet_sprite.get_texture()
            .expect("Unable to get Texture")
            .get_width() as f32 * 0.5;
        let scale = rng.gen_range(0.5, 2.5) * planet_sprite.get_scale().x;
        let scale_vector = Vector2::new(scale, scale);
        planet_area.set_scale(scale_vector);

        props.radius = 0.42 * scale * size;
        let diameter = 2.0 * props.radius;
        let x_offset = (rng.gen_range(0.0, 1.0) * viewport_width).clamp(diameter, viewport_width - diameter);
        let y_offset = (rng.gen_range(0.0, 1.0) * viewport_height).clamp(diameter, viewport_height - diameter);
        owner.set_position(Vector2::new(x_offset, y_offset));
    }

    pub fn set_id(&self, id: usize) {
        let mut props = self.properties.borrow_mut();
        props.id = id;
    }

    pub fn get_main_loop(&self) -> Ref<MainLoop<Starmap2D, Player2D>> {
        self.main_loop.as_ref().unwrap().borrow()
    }

    pub fn set_input_handler<F: 'static>(&mut self, input_handler: Rc<RefCell<InputHandler2D>>, input_handler_fn: F) 
    where 
        F: Fn(Box<&Planet>, PlayerAction) -> ()
    {
        self.input_handler = Some(input_handler);
        self.input_handler_fn = Some(Box::new(input_handler_fn));
    }

    pub fn set_main_loop(&mut self, main_loop: Rc<RefCell<MainLoop<Starmap2D, Player2D>>>) {
        self.main_loop = Some(main_loop);
    }

    pub fn set_resources(&self, initial: f32, inc: f32) {
        let mut props = self.properties.borrow_mut();
        self.business.resources_init(&mut props, initial, inc);
    }

    pub unsafe fn with_mut<F, T>(node: Node2D, mut with_fn: F) -> T
    where
        F: FnMut(&mut Planet) -> T
    {
        let instance = Instance::<Planet>::try_from_base(node).unwrap();
        instance.map_mut(|class, _owner| with_fn(class)).unwrap()
    }

    pub unsafe fn with<F, T>(node: Node2D, with_fn: F) -> T
    where
        F: Fn(&Planet) -> T
    {
        let instance = Instance::<Planet>::try_from_base(node).unwrap();
        instance.map(|class, _owner| with_fn(class)).unwrap()
    }

    pub unsafe fn get_by_id(planets: &Vec<Rc<Node2D>>, id: usize) -> &Rc<Node2D> {
        planets.iter()
        .find(|p| {
            Planet::with(***p, |planet| planet.properties().id == id)
        }).unwrap()
    }

}