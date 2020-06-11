pub mod player;
pub mod starmap;

use gdnative::GodotObject;

use self::starmap::Starmap;
use self::player::Player;

pub struct MainLoop<T: GodotObject, S: GodotObject> {
    starmap: Option<Starmap<T>>,
    players: Vec<Player<T, S>>,
}

impl <T: GodotObject, S: GodotObject> MainLoop<T, S> {

    pub fn new() -> MainLoop<T, S> {
        MainLoop {
            starmap: None,
            players: vec!(),
        }
    }

    pub fn add_player(&mut self, player: Player<T, S>) {
        self.players.push(player);
    }

    pub fn set_starmap(&mut self, starmap: Starmap<T>) {
        self.starmap = Some(starmap);
    }
}