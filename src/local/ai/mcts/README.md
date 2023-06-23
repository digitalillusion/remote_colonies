# mcts

This is a library for Monte Carlo tree search (Mcts) in Rust.

The implementation is parallel and lock-free. The generic design allows it to be used in a wide variety of domains. It can accomodate different search methodologies (for example, rollout-based or neural-net-based leaf evaluation).

[Documentation](https://docs.rs/mcts)
