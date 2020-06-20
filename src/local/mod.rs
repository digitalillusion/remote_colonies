pub mod model;
pub mod player;
pub mod starmap;
pub mod planet;
pub mod input;

use gdnative::*;

use std::rc::Rc;

use self::starmap::Starmap;
use self::player::*;


pub struct MainLoop<T,U>
where 
    T: GodotObject,
    U: GodotObject,
{
    pub starmap: Option<Starmap<T>>,
    pub players: Vec<Rc<Player<T, U>>>
}

impl <T, U> MainLoop<T, U> where 
    T: GodotObject,
    U: GodotObject,
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

    pub fn get_current_player(&self) -> &Rc<Player<T, U>> {
        self.players.get(0).unwrap()
    }

    pub fn run(&self) {
        
    }
}