use mcts::transposition_table::*;
use mcts::tree_policy::*;
use mcts::*;

use super::AiState;
use crate::local::model::*;
use crate::local::player::*;

pub struct MyEvaluator;

impl Evaluator<MyMCTS> for MyEvaluator {
    type StateEvaluation = i64;

    fn evaluate_new_state(
        &self,
        state: &AiState,
        moves: &Vec<PlayerAction>,
        _: Option<SearchHandle<MyMCTS>>,
    ) -> (Vec<()>, i64) {
        (vec![(); moves.len()], state.metrics.evaluate())
    }
    fn interpret_evaluation_for_player(&self, evaln: &i64, _player: &ContenderProperties) -> i64 {
        *evaln
    }
    fn evaluate_existing_state(&self, _: &AiState, evaln: &i64, _: SearchHandle<MyMCTS>) -> i64 {
        *evaln
    }
}

#[derive(Default)]
pub struct MyMCTS;

impl MCTS for MyMCTS {
    type State = AiState;
    type Eval = MyEvaluator;
    type NodeData = ();
    type ExtraThreadData = ();
    type TreePolicy = UCTPolicy;
    type TranspositionTable = ApproxTable<Self>;

    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        CycleBehaviour::UseCurrentEvalWhenCycleDetected
    }
}
