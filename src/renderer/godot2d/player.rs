use gdnative::prelude::*;

use std::cell::*;

use super::planet::RefPlanetNode2D;
use super::ship::{RefShipNode2D, Ship};
use crate::local::model::*;
use crate::local::player::*;

pub struct Player2D {
    pub planets: RefCell<Vec<RefPlanetNode2D>>,
    pub ships: RefCell<Vec<RefShipNode2D>>,

    properties: RefCell<ContenderProperties>,
}

impl Contender for Player2D {
    fn properties(&self) -> ContenderProperties {
        *self.properties.borrow()
    }
}

impl Player for Player2D {
    type CelestialType = RefPlanetNode2D;
    type VesselType = RefShipNode2D;

    fn new(id: usize, planet: RefPlanetNode2D, ship: RefShipNode2D, bot: bool) -> Self {
        let properties = ContenderProperties {
            id,
            color: get_color(id),
            bot,
        };
        Player2D {
            properties: RefCell::new(properties),
            planets: RefCell::new(vec![planet]),
            ships: RefCell::new(vec![ship]),
        }
    }

    fn destroy(&self) {
        self.ships
            .borrow_mut()
            .iter_mut()
            .for_each(|s| unsafe { s.assume_safe() }.queue_free());
    }

    fn add_ship(&self, ship: RefShipNode2D) {
        self.ships.borrow_mut().push(ship);
    }

    fn is_playing(&self) -> bool {
        self.ships.borrow().len() > 0
    }

    fn get_ships_on_planet(&self, planet_props: CelestialProperties) -> Vec<VesselProperties> {
        let player_ships = self.ships.borrow();
        let player_ships_on_planet: Vec<VesselProperties> = player_ships
            .iter()
            .filter_map(|ship_node| {
                Ship::with(ship_node, |ship| {
                    if ship.properties().celestial_id == planet_props.id {
                        return Some(ship.properties());
                    }
                    None
                })
            })
            .collect();

        player_ships_on_planet
    }
}

fn get_color(id: usize) -> Color {
    let colors = [
        (230, 25, 75),   // Red
        (60, 180, 75),   // Green
        (255, 225, 25),  // Yellow
        (0, 130, 200),   // Blue
        (245, 130, 48),  // Orange
        (145, 30, 180),  // Purple
        (70, 240, 240),  // Cyan
        (240, 50, 230),  // Magenta
        (210, 245, 60),  // Lime
        (250, 190, 212), // Pink
        (0, 128, 128),   // Teal
        (220, 190, 255), // Lavender
        (170, 110, 40),  // Brown
        (255, 250, 200), // Beige
        (128, 0, 0),     // Maroon
        (170, 255, 195), // Mint
        (128, 128, 0),   // Olive
        (255, 215, 180), // Apricot
        (0, 0, 128),     // Navy
        (128, 128, 128), // Grey
    ];
    Color::from_rgb(
        colors[id].0 as f32 / 255.0,
        colors[id].1 as f32 / 255.0,
        colors[id].2 as f32 / 255.0,
    )
}
