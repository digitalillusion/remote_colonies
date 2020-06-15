pub mod model;
pub mod player;
pub mod starmap;
pub mod planet;
pub mod input;

use gdnative::*;

use std::rc::Rc;

use self::starmap::Starmap;
use self::player::*;


pub struct MainLoop<T>
where 
    T: GodotObject
{
    pub starmap: Option<Starmap<T>>,
    pub players: Vec<Rc<Player<T>>>
}

impl <T> MainLoop<T> where 
    T: GodotObject
{
    pub fn new() -> MainLoop<T> {
        MainLoop {
            starmap: None,
            players: vec!(),
        }
    }

    pub fn set_starmap(&mut self, starmap: Starmap<T>) {
        self.starmap = Some(starmap);
    }  

    pub fn run(&self) {
        
    }
}