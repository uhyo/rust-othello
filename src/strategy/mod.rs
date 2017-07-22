// Othello playing strategy.
use rand;
use rand::distributions;
use rand::distributions::IndependentSample;

use board::{Board, Move};
use options::Opts;

pub mod util;
pub mod book;
pub mod search;
pub mod ending;

use self::book::Book;
use self::search::Searcher;
use self::ending::EndingSearcher;


pub trait Strategy {
    fn play<B>(&mut self, board: &B, last_move: Option<Move>, time: i32) -> Move where B: Board + Clone;
    fn reset(&mut self);
}

pub struct RandomStrategy {
    // 打つ候補の列
    points: Vec<(u8, u8)>,
}
impl RandomStrategy{
    fn new () -> Self{
        RandomStrategy {
            points: make_points(),
        }
    }
}
fn make_points() -> Vec<(u8, u8)> {
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
    return points;
}

impl Strategy for RandomStrategy{
    fn play<B>(&mut self, board: &B, _last_move: Option<Move>, _time: i32) -> Move
        where B: Board + Clone {
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
    fn reset(&mut self) {
        self.points = make_points();
    }
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum MainStrategyState {
    Book,
    Search,
    Ending,
    Random,
}
pub struct MainStrategy {
    state: MainStrategyState,
    random: RandomStrategy,
    book: Book,
    searcher: Searcher,
    ending: EndingSearcher,
    // options
    ending_turns: u32,
}
impl MainStrategy {
    fn new (opts: &Opts) -> Self {
        let random = RandomStrategy::new();
        let book = Book::new();
        let searcher = Searcher::new();
        let ending = EndingSearcher::new();

        MainStrategy {
            state: MainStrategyState::Book,
            random,
            book,
            searcher,
            ending,
            ending_turns: opts.ending_turns,
        }
    }
}
impl Strategy for MainStrategy {
    fn play<B>(&mut self, board: &B, last_move: Option<Move>, time: i32) -> Move 
        where B: Board + Clone {
        if self.state == MainStrategyState::Book {
            match self.book.gen(last_move) {
                None => {
                    self.state = MainStrategyState::Search;
                },
                Some((mv2, flg)) => {
                    if !flg {
                        // もう定石が無い
                        self.state = MainStrategyState::Search;
                    }
                    trace!("Using opening book");
                    return mv2;
                }
            }
        }
        if self.state == MainStrategyState::Search {
            if board.count_both() >= 64 - self.ending_turns {
                // 終盤を読み切る
                self.state = MainStrategyState::Ending;
            } else {
                trace!("Using searching strategy");
                return self.searcher.search(board);
            }
        }
        if self.state == MainStrategyState::Ending {
            trace!("Using ending strategy");
            return self.ending.search(board, board.get_turn(), last_move.unwrap());
        }
        trace!("Using random strategy");
        return self.random.play(board, last_move, time);
    }
    fn reset(&mut self) {
        self.state = MainStrategyState::Book;
        self.book.reset();
        self.searcher.reset();
        self.ending.reset();
    }
}

pub fn make_strategy(opts: &Opts) -> MainStrategy {
    MainStrategy::new(opts)
}

