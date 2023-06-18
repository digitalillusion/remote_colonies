use crate::local::ai::mcts::transposition_table::ApproxTable;
use crate::local::ai::mcts::tree_policy::UCTPolicy;
use crate::local::ai::mcts::{CycleBehaviour, Evaluator, Mcts, SearchHandle};
use crate::local::ai::AiState;
use crate::local::model::*;
use crate::local::player::*;

pub struct MyEvaluator;

impl Evaluator<MyMcts> for MyEvaluator {
    type StateEvaluation = i64;

    fn evaluate_new_state(
        &self,
        state: &AiState,
        moves: &Vec<PlayerAction>,
        _: Option<SearchHandle<MyMcts>>,
    ) -> (Vec<()>, i64) {
        (vec![(); moves.len()], state.metrics.evaluate())
    }
    fn interpret_evaluation_for_player(&self, evaln: &i64, _player: &ContenderProperties) -> i64 {
        *evaln
    }
    fn evaluate_existing_state(&self, _: &AiState, evaln: &i64, _: SearchHandle<MyMcts>) -> i64 {
        *evaln
    }
}

#[derive(Default)]
pub struct MyMcts;

impl Mcts for MyMcts {
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
