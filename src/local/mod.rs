pub mod model;
pub mod player;
pub mod starmap;

use gdnative::GodotObject;

use std::thread::JoinHandle;

use self::starmap::Starmap;
use self::player::Player;


pub struct MainLoop<T>
where 
    T: GodotObject
{
    starmap: Option<Starmap<T>>,
    players: Vec<Player<T>>,
    threads: Vec<JoinHandle<()>>,
}

impl <T> MainLoop<T> where 
    T: GodotObject
{
    pub fn new() -> MainLoop<T> {
        MainLoop {
            starmap: None,
            players: vec!(),
            threads: vec!(),
        }
    }

    pub fn add_player(&mut self, player: Player<T>) {
        self.players.push(player);
    }

    pub fn set_starmap(&mut self, starmap: Starmap<T>) {
        self.starmap = Some(starmap);
    }

    pub fn add_thread(&mut self, join_handle: JoinHandle<()>) {
        self.threads.push(join_handle);
    }

    pub fn run(&self) {
        
    }
}