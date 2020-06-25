use std::sync::*;

use mcts::*;
use mcts::tree_policy::*;
use mcts::transposition_table::*;

use super::model::*;

#[derive(Clone, Debug)]
pub struct AiState {
    player_properties: ContenderProperties
}

impl AiState {
    fn new(player: &dyn Contender) -> Self {
        AiState {
            player_properties: player.properties()
        }
    }

    pub fn refresh_facts() {
        
    }
}

impl GameState for AiState {
    type Move = ContenderAction;
    type Player = ContenderProperties;
    type MoveList = Vec<ContenderAction>;
 
    fn current_player(&self) -> Self::Player {
        self.player_properties
    }
    fn available_moves(&self) -> Vec<ContenderAction> {
        let moves = vec![ContenderAction::Wait];

        moves
    }
    fn make_move(&mut self, mov: &Self::Move) {
        match *mov {
            _ => ()
        }
    }
}