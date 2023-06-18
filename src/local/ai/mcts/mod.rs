#![allow(dead_code)]

//! This is a library for Monte Carlo tree search.
//!
//! It is still under development and the documentation isn't good. However, the following example may be helpful:
//!
//! ```
//! use mcts::*;
//! use mcts::tree_policy::*;
//! use mcts::transposition_table::*;
//!
//! // A really simple game. There's one player and one number. In each move the player can
//! // increase or decrease the number. The player's score is the number.
//! // The game ends when the number reaches 100.
//! //
//! // The best strategy is to increase the number at every step.
//!
//! #[derive(Clone, Debug, PartialEq)]
//! struct CountingGame(i64);
//!
//! #[derive(Clone, Debug, PartialEq)]
//! enum Move {
//!     Add, Sub
//! }
//!
//! impl GameState for CountingGame {
//!     type Move = Move;
//!     type Player = ();
//!     type MoveList = Vec<Move>;
//!
//!     fn current_player(&self) -> Self::Player {
//!         ()
//!     }
//!     fn available_moves(&self) -> Vec<Move> {
//!         let x = self.0;
//!         if x == 100 {
//!             vec![]
//!         } else {
//!             vec![Move::Add, Move::Sub]
//!         }
//!     }
//!     fn make_move(&mut self, mov: &Self::Move) {
//!         match *mov {
//!             Move::Add => self.0 += 1,
//!             Move::Sub => self.0 -= 1,
//!         }
//!     }
//! }
//!
//! impl TranspositionHash for CountingGame {
//!     fn hash(&self) -> u64 {
//!         self.0 as u64
//!     }
//! }
//!
//! struct MyEvaluator;
//!
//! impl Evaluator<MyMcts> for MyEvaluator {
//!     type StateEvaluation = i64;
//!
//!     fn evaluate_new_state(&self, state: &CountingGame, moves: &Vec<Move>,
//!         _: Option<SearchHandle<MyMcts>>)
//!         -> (Vec<()>, i64) {
//!         (vec![(); moves.len()], state.0)
//!     }
//!     fn interpret_evaluation_for_player(&self, evaln: &i64, _player: &()) -> i64 {
//!         *evaln
//!     }
//!     fn evaluate_existing_state(&self, _: &CountingGame,  evaln: &i64, _: SearchHandle<MyMcts>) -> i64 {
//!         *evaln
//!     }
//! }
//!
//! #[derive(Default)]
//! struct MyMcts;
//!
//! impl Mcts for MyMcts {
//!     type State = CountingGame;
//!     type Eval = MyEvaluator;
//!     type NodeData = ();
//!     type ExtraThreadData = ();
//!     type TreePolicy = UCTPolicy;
//!     type TranspositionTable = ApproxTable<Self>;
//!
//!     fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
//!         CycleBehaviour::UseCurrentEvalWhenCycleDetected
//!     }
//! }
//!
//! let game = CountingGame(0);
//! let mut mcts = MctsManager::new(game, MyMcts, MyEvaluator, UCTPolicy::new(0.5),
//!     ApproxTable::new(1024));
//! mcts.playout_n_parallel(10000, 4); // 10000 playouts, 4 search threads
//! mcts.tree().debug_moves();
//! assert_eq!(mcts.best_move().unwrap(), Move::Add);
//! assert_eq!(mcts.principal_variation(50),
//!     vec![Move::Add; 50]);
//! assert_eq!(mcts.principal_variation_states(5),
//!     vec![
//!         CountingGame(0),
//!         CountingGame(1),
//!         CountingGame(2),
//!         CountingGame(3),
//!         CountingGame(4),
//!         CountingGame(5)]);
//! ```

mod atomics;
mod search_tree;
pub mod transposition_table;
pub mod tree_policy;

pub use search_tree::*;
use transposition_table::*;
use tree_policy::*;

use atomics::*;
use std::sync::Arc;
use std::thread::JoinHandle;

pub trait Mcts: Sized + Sync {
    type State: GameState + Sync;
    type Eval: Evaluator<Self>;
    type TreePolicy: TreePolicy<Self>;
    type NodeData: Default + Sync + Send;
    type TranspositionTable: TranspositionTable<Self>;
    type ExtraThreadData;

    fn virtual_loss(&self) -> i64 {
        0
    }
    fn visits_before_expansion(&self) -> u64 {
        1
    }
    fn node_limit(&self) -> usize {
        std::usize::MAX
    }
    fn select_child_after_search<'a>(&self, children: &'a [MoveInfo<Self>]) -> &'a MoveInfo<Self> {
        children.iter().max_by_key(|child| child.visits()).unwrap()
    }
    /// `playout` panics when this length is exceeded. Defaults to one million.
    fn max_playout_length(&self) -> usize {
        1_000_000
    }
    fn on_backpropagation(&self, _evaln: &StateEvaluation<Self>, _handle: SearchHandle<Self>) {}
    fn cycle_behaviour(&self) -> CycleBehaviour<Self> {
        if std::mem::size_of::<Self::TranspositionTable>() == 0 {
            CycleBehaviour::Ignore
        } else {
            CycleBehaviour::PanicWhenCycleDetected
        }
    }
}

pub struct ThreadData<Spec: Mcts> {
    pub policy_data: TreePolicyThreadData<Spec>,
    pub extra_data: Spec::ExtraThreadData,
}

impl<Spec: Mcts> Default for ThreadData<Spec>
where
    TreePolicyThreadData<Spec>: Default,
    Spec::ExtraThreadData: Default,
{
    fn default() -> Self {
        Self {
            policy_data: Default::default(),
            extra_data: Default::default(),
        }
    }
}

pub type MoveEvaluation<Spec> = <<Spec as Mcts>::TreePolicy as TreePolicy<Spec>>::MoveEvaluation;
pub type StateEvaluation<Spec> = <<Spec as Mcts>::Eval as Evaluator<Spec>>::StateEvaluation;
pub type Move<Spec> = <<Spec as Mcts>::State as GameState>::Move;
pub type MoveList<Spec> = <<Spec as Mcts>::State as GameState>::MoveList;
pub type Player<Spec> = <<Spec as Mcts>::State as GameState>::Player;
pub type TreePolicyThreadData<Spec> =
    <<Spec as Mcts>::TreePolicy as TreePolicy<Spec>>::ThreadLocalData;

pub trait GameState: Clone {
    type Move: Sync + Send + Clone;
    type Player: Sync;
    type MoveList: std::iter::IntoIterator<Item = Self::Move>;

    fn current_player(&self) -> Self::Player;
    fn available_moves(&self) -> Self::MoveList;
    fn make_move(&mut self, mov: &Self::Move);
}

pub trait Evaluator<Spec: Mcts>: Sync {
    type StateEvaluation: Sync + Send;

    fn evaluate_new_state(
        &self,
        state: &Spec::State,
        moves: &MoveList<Spec>,
        handle: Option<SearchHandle<Spec>>,
    ) -> (Vec<MoveEvaluation<Spec>>, Self::StateEvaluation);

    fn evaluate_existing_state(
        &self,
        state: &Spec::State,
        existing_evaln: &Self::StateEvaluation,
        handle: SearchHandle<Spec>,
    ) -> Self::StateEvaluation;

    fn interpret_evaluation_for_player(
        &self,
        evaluation: &Self::StateEvaluation,
        player: &Player<Spec>,
    ) -> i64;
}

pub struct MctsManager<Spec: Mcts> {
    search_tree: SearchTree<Spec>,
    // thread local data when we have no asynchronous workers
    single_threaded_tld: Option<ThreadData<Spec>>,
    print_on_playout_error: bool,
}

impl<Spec: Mcts> MctsManager<Spec>
where
    ThreadData<Spec>: Default,
{
    pub fn new(
        state: Spec::State,
        manager: Spec,
        eval: Spec::Eval,
        tree_policy: Spec::TreePolicy,
        table: Spec::TranspositionTable,
    ) -> Self {
        let search_tree = SearchTree::new(state, manager, tree_policy, eval, table);
        let single_threaded_tld = None;
        Self {
            search_tree,
            single_threaded_tld,
            print_on_playout_error: true,
        }
    }

    pub fn print_on_playout_error(&mut self, v: bool) -> &mut Self {
        self.print_on_playout_error = v;
        self
    }

    pub fn playout(&mut self) {
        // Avoid overhead of thread creation
        if self.single_threaded_tld.is_none() {
            self.single_threaded_tld = Some(Default::default());
        }
        self.search_tree
            .playout(self.single_threaded_tld.as_mut().unwrap());
    }
    pub fn playout_until<Predicate: FnMut() -> bool>(&mut self, mut pred: Predicate) {
        while !pred() {
            self.playout();
        }
    }
    pub fn playout_n(&mut self, n: u64) {
        for _ in 0..n {
            self.playout();
        }
    }
    pub fn principal_variation_info(&self, num_moves: usize) -> Vec<MoveInfoHandle<Spec>> {
        self.search_tree.principal_variation(num_moves)
    }
    pub fn principal_variation(&self, num_moves: usize) -> Vec<Move<Spec>> {
        self.search_tree
            .principal_variation(num_moves)
            .into_iter()
            .map(|x| x.get_move().clone())
            .collect()
    }
    pub fn principal_variation_states(&self, num_moves: usize) -> Vec<Spec::State> {
        let moves = self.principal_variation(num_moves);
        let mut states = vec![self.search_tree.root_state().clone()];
        for mov in moves {
            let mut state = states[states.len() - 1].clone();
            state.make_move(&mov);
            states.push(state);
        }
        states
    }
    pub fn tree(&self) -> &SearchTree<Spec> {
        &self.search_tree
    }
    pub fn best_move(&self) -> Option<Move<Spec>> {
        self.principal_variation(1).get(0).cloned()
    }
    pub fn reset(self) -> Self {
        Self {
            search_tree: self.search_tree.reset(),
            print_on_playout_error: self.print_on_playout_error,
            single_threaded_tld: None,
        }
    }
}

// https://stackoverflow.com/questions/26998485/rust-print-format-number-with-thousand-separator
fn thousands_separate(x: usize) -> String {
    let s = format!("{x}");
    let bytes: Vec<_> = s.bytes().rev().collect();
    let chunks: Vec<_> = bytes
        .chunks(3)
        .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap())
        .collect();
    let result: Vec<_> = chunks.join(",").bytes().rev().collect();
    String::from_utf8(result).unwrap()
}

#[must_use]
pub struct AsyncSearch<'a, Spec: 'a + Mcts> {
    manager: &'a mut MctsManager<Spec>,
    stop_signal: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl<'a, Spec: Mcts> AsyncSearch<'a, Spec> {
    pub fn halt(self) {}
    pub fn num_threads(&self) -> usize {
        self.threads.len()
    }
}

impl<'a, Spec: Mcts> Drop for AsyncSearch<'a, Spec> {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        drain_join_unwrap(&mut self.threads);
    }
}

#[must_use]
pub struct AsyncSearchOwned<Spec: Mcts> {
    manager: Option<Box<MctsManager<Spec>>>,
    stop_signal: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl<Spec: Mcts> AsyncSearchOwned<Spec> {
    fn stop_threads(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        drain_join_unwrap(&mut self.threads);
    }
    pub fn halt(mut self) -> MctsManager<Spec> {
        self.stop_threads();
        *self.manager.take().unwrap()
    }
    pub fn num_threads(&self) -> usize {
        self.threads.len()
    }
}

impl<Spec: Mcts> Drop for AsyncSearchOwned<Spec> {
    fn drop(&mut self) {
        self.stop_threads();
    }
}

impl<Spec: Mcts> From<MctsManager<Spec>> for AsyncSearchOwned<Spec> {
    /// An `MctsManager` is an `AsyncSearchOwned` with zero threads searching.
    fn from(m: MctsManager<Spec>) -> Self {
        Self {
            manager: Some(Box::new(m)),
            stop_signal: Arc::new(AtomicBool::new(false)),
            threads: Vec::new(),
        }
    }
}

fn drain_join_unwrap(threads: &mut Vec<JoinHandle<()>>) {
    let join_results: Vec<_> = threads.drain(..).map(|x| x.join()).collect();
    for x in join_results {
        x.unwrap();
    }
}

pub enum CycleBehaviour<Spec: Mcts> {
    Ignore,
    UseCurrentEvalWhenCycleDetected,
    PanicWhenCycleDetected,
    UseThisEvalWhenCycleDetected(StateEvaluation<Spec>),
}
