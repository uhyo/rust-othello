// Othello playing strategy.
use rand;
use rand::distributions;
use rand::distributions::IndependentSample;

use board::{Board, Tile, Move, turn_to_tile};


pub trait Strategy {
    fn play(&mut self, board: &Board, time: i32) -> Move;
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
    fn play(&mut self, board: &Board, _time: i32) -> Move{
        let mut points = self.points.iter();
        // 候補を順番に試す
        while let Some(&(x, y)) = points.next() {
            if putable(board, x, y) {
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

pub fn make_strategy() -> RandomStrategy{
    RandomStrategy::new()
}

// この場所に石を置けるか?
fn putable(board: &Board, x: u8, y: u8) -> bool{
    if board.get(x, y) != Tile::Empty {
        // 置けない
        return false;
    }
    let (me, op) = turn_to_tile(board.get_turn());
    lazy_static! {
        static ref DVEC: Vec<(i32, i32)> = vec![
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
    }
    // 8方向を試す
    for &(dx, dy) in DVEC.iter() {
        let mut flag = false;
        let mut cx = x as i32;
        let mut cy = y as i32;
        loop {
            cx += dx;
            cy += dy;
            if cx < 0 || cx > 7 || cy < 0 || cy > 7 {
                break;
            }
            let t = board.get(cx as u8, cy as u8);
            if !flag {
                if t != op {
                    // 取れないわ
                    break;
                }
                flag = true;
            } else {
                if t == Tile::Empty {
                    // 取れなかった
                    break;
                }
                if t == me {
                    // 取れるわ
                    return true;
                }
            }
        }
    }
    return false;
}
