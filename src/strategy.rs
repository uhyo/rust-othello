// Othello playing strategy.
use board::{Board, Tile};

#[derive(Copy, Clone)]
pub enum Play {
    Pass,
    Put(u32, u32, Tile),
}
pub trait Strategy {
    fn play(&mut self, board: &Board) -> Play;
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
    fn play(&mut self, board: &Board) -> Play{
        Play::Pass
    }
}

pub fn make_strategy() -> RandomStrategy{
    RandomStrategy::new()
}
