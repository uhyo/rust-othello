// Othello playing strategy.
use rand;
use rand::distributions;
use rand::distributions::IndependentSample;

use board::{Board, Tile, Move, turn_to_tile};

mod util;
mod book;
use self::book::Book;


pub trait Strategy {
    fn play(&mut self, board: &Board, last_move: Option<Move>, time: i32) -> Move;
}

pub struct RandomStrategy {
    // 打つ候補の列
    points: Vec<(u8, u8)>,
}
impl RandomStrategy{
    fn new () -> Self{
        let mut points: Vec<(u8, u8)> = Vec::with_capacity(64);
        // init sequence
        for i in 0..64 {
            points.push(((i as u8) % 8, (i as u8) / 8));
        }
        // shuffle
        let mut rng = rand::thread_rng();
        for i in 0..64 {
            let ran = distributions::Range::new(i, 64);
            let j = ran.ind_sample(&mut rng);
            // 入れ替える
            let tmp = points[i];
            points[i] = points[j];
            points[j] = tmp;
        }

        RandomStrategy {
            points,
        }
    }
}

impl Strategy for RandomStrategy{
    fn play(&mut self, board: &Board, _last_move: Option<Move>, _time: i32) -> Move{
        let mut points = self.points.iter();
        // 候補を順番に試す
        while let Some(&(x, y)) = points.next() {
            if util::putable(board, x, y) {
                return Move::Put {
                    x,
                    y,
                };
            }
        }
        // いけそうなところがなかった
        return Move::Pass;
    }
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum MainStrategyState {
    Book,
    Random,
}
pub struct MainStrategy {
    state: MainStrategyState,
    random: RandomStrategy,
    book: Book,
}
impl MainStrategy {
    fn new () -> Self {
        let random = RandomStrategy::new();
        let book = Book::new();

        MainStrategy {
            state: MainStrategyState::Book,
            random,
            book,
        }
    }
}
impl Strategy for MainStrategy {
    fn play(&mut self, board: &Board, last_move: Option<Move>, time: i32) -> Move{
        if self.state == MainStrategyState::Book {
            match self.book.gen(last_move) {
                None => {
                    self.state = MainStrategyState::Random;
                },
                Some((mv2, flg)) => {
                    if !flg {
                        // もう定石が無い
                        self.state = MainStrategyState::Random;
                    }
                    trace!("Using opening book");
                    return mv2;
                }
            }
        }
        trace!("Using random strategy");
        return self.random.play(board, last_move, time);
    }
}

pub fn make_strategy() -> MainStrategy{
    MainStrategy::new()
}

