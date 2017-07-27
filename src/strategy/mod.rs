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
    // playをひとつ進める
    fn play<B>(&mut self, board: &B, last_move: Option<Move>, time: i32) -> Move where B: Board + Clone;
    // strategyは使わないが手が進んだ場合
    fn go<B>(&mut self, board: &B, last_move_op: Option<Move>, last_move_me: Move) where B: Board + Clone;
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
            if util::putable(board, x, y, board.get_turn()) {
                return Move::Put {
                    x,
                    y,
                };
            }
        }
        // いけそうなところがなかった
        return Move::Pass;
    }
    fn go<B>(&mut self, _board: &B, _last_move_op: Option<Move>, _last_move_me: Move) {
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
        let searcher = Searcher::new(opts);
        let ending = EndingSearcher::new(opts);

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
            match self.book.gen(board.get_turn(), last_move) {
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
    fn go<B>(&mut self, _board: &B, last_move_op: Option<Move>, last_move_me: Move) {
        if self.state == MainStrategyState::Book {
            let r = 
                if let Some(m) = last_move_op {
                    self.book.go(m)
                } else {
                    true
                };
            if !r {
                self.state = MainStrategyState::Search;
            } else {
                let r2 = self.book.go(last_move_me);
                if !r2 {
                    self.state = MainStrategyState::Search;
                }
            }
        } else if self.state == MainStrategyState::Search {
            if let Some(m) = last_move_op {
                self.searcher.go(m);
            }
            self.searcher.go(last_move_me);
        } else if self.state == MainStrategyState::Ending {
            if let Some(m) = last_move_op {
                self.ending.go(m);
            }
            self.ending.go(last_move_me);
        }
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

pub struct SometimesRandomStrategy {
    main: MainStrategy,
    random: RandomStrategy,
    rng: Box<rand::Rng>,
}
impl SometimesRandomStrategy {
    fn new(opts: &Opts) -> Self {
        SometimesRandomStrategy {
            main: MainStrategy::new(opts),
            random: RandomStrategy::new(),
            rng: Box::new(rand::thread_rng()),
        }
    }
}
impl Strategy for SometimesRandomStrategy {
    fn reset(&mut self) {
        self.main.reset();
        self.random.reset();
    }
    fn go<B>(&mut self, board: &B, last_move_op: Option<Move>, last_move_me: Move) where B: Board + Clone {
        self.main.go(board, last_move_op, last_move_me);
        self.random.go(board, last_move_op, last_move_me);
    }
    fn play<B>(&mut self, board: &B, last_move: Option<Move>, time: i32) -> Move
        where B: Board + Clone {
        // たまにrandomを選ぶ
        let ran = distributions::Range::new(0, 100);
        let i = ran.ind_sample(&mut self.rng);
        let rn =
            if self.main.state == MainStrategyState::Book {
                // 序盤は大きめのランダムさ
                25
            } else if self.main.state == MainStrategyState::Search {
                9
            } else {
                // 終盤は間違える必要がない
                0
            };
        if i < rn {
            // ランダムな確率
            trace!("Using random strategy");
            let mv = self.random.play(board, last_move, time);
            self.main.go(board, last_move, mv);
            return mv;
        } else {
            let mv = self.main.play(board, last_move, time);
            self.random.go(board, last_move, mv);
            return mv;
        }
    }
}

pub fn make_learner(opts: &Opts) -> SometimesRandomStrategy {
    SometimesRandomStrategy::new(opts)
}
