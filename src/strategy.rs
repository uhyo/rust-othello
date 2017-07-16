// Othello playing strategy.
use board::{Board, Tile, Move};

pub trait Strategy {
    fn play(&mut self, board: &Board) -> Move;
}

pub struct RandomStrategy {
}
impl RandomStrategy{
    fn new () -> Self{
        RandomStrategy {}
    }
}

impl Strategy for RandomStrategy{
    // TODO
    fn play(&mut self, board: &Board) -> Move{
        Move::Pass
    }
}

pub fn make_strategy() -> RandomStrategy{
    RandomStrategy::new()
}
