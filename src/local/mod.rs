pub mod model;
pub mod player;
pub mod starmap;
pub mod planet;
pub mod input;

use gdnative::*;

use std::rc::Rc;

use self::starmap::Starmap;
use self::player::*;
use crate::local::model::*;


pub struct MainLoop<T,U>
where 
    T: GodotObject,
    U: Player,
{
    pub starmap: Option<Starmap<T>>,
    pub players: Vec<Rc<U>>
}

impl <T, U> MainLoop<T, U> where 
    T: GodotObject,
    U: Player,
{
    pub fn new() -> MainLoop<T, U> {
        MainLoop {
            starmap: None,
            players: vec!(),
        }
    }

    pub fn set_starmap(&mut self, starmap: Starmap<T>) {
        self.starmap = Some(starmap);
    }  

    pub fn get_current_player(&self) -> &Rc<U> {
        self.players.get(0).unwrap()
    }

    pub unsafe fn get_ships_by_player(&self, planet_props: CelestialProperties) -> Vec<(ContenderProperties, Vec<VesselProperties>)> {
        let mut ships_by_player: Vec<(ContenderProperties, Vec<VesselProperties>)> = vec!();
        self.players.iter().for_each(|player| {
            let ships_on_planet = player.get_ships_on_planet(planet_props);
            let tuple = (player.properties(), ships_on_planet);
            ships_by_player.push(tuple);
        });
        ships_by_player
    }

    pub fn run(&self) {
        
    }
}