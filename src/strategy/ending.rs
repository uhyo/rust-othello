// Seach for ending
use std::cmp::{min, max, Ordering};
use board::{Board, Move, Turn, turn_to_tile};
use strategy::util::iter_moves;

pub struct GameTree {
    // 現在のポジションの最大値と最小値
    max: Ordering,
    min: Ordering,
    moves: Box<Vec<(Move, GameTree)>>,
}
impl GameTree {
    // 指定された手で進む
    pub fn take(mut self, mv: Move) -> Self {
        let mut idx = 0;
        for (i, &(mvv, _)) in self.moves.iter().enumerate() {
            if mv == mvv {
                // これだ
                idx = i;
            }
        }
        return self.moves.swap_remove(idx).1;
    }
}

pub struct EndingSearcher {
    table: Option<GameTree>,
}
impl EndingSearcher {
    pub fn new() -> EndingSearcher {
        EndingSearcher {
            table: None,
        }
    }
    pub fn reset(&mut self) {
        self.table = None;
    }
    pub fn search<B>(&mut self, board: &B, mycolor: Turn, last_move: Move) -> Move where B: Board + Clone {
        let t = self.table.take();
        let table =
            match t {
                Some(tt) => {
                    tt.take(last_move)
                },
                None => {
                    // ゲームの終わりまでサーチする
                    search(board, mycolor, false)
                }
            };
        match table.moves.iter().next() {
            None => {
                // 動けない
                self.table = Some(table);
                return Move::Pass;
            },
            Some(&(mv, _)) => {
                self.table = Some(table.take(mv));
                trace!("Choose {:?}/{:?} move", self.table.as_ref().unwrap().max, self.table.as_ref().unwrap().min);
                return mv;
            },
        }
    }
}

fn search<B>(board: &B, mycolor: Turn, one_pass: bool) -> GameTree where B: Board + Clone {
    let mut flg = false;
    let mut rmin = Ordering::Greater;
    let mut rmax = Ordering::Less;
    let mut moves = Vec::new();
    for mv in iter_moves(board) {
        flg = true;

        if let Move::Put {x: _, y: _} = mv {
            // 次の盤面
            let mut board2 = board.clone();
            board2.apply_move(mv).unwrap();
            let t = search(&mut board2, mycolor, false);
            rmin = min(rmin, t.min);
            rmax = max(rmax, t.max);
            moves.push((mv, t));
        }
    }
    // 有利なものを先頭へ
    moves.sort_by(|&(_, ref a), &(_, ref b)| b.max.cmp(&a.max).then(b.min.cmp(&a.min)));
    if !flg {
        // 置けるところがない
        if one_pass {
            // Double Pass!
            // 終局っぽいので数を数える
            let (me, op) = turn_to_tile(mycolor);
            let mycount = board.count(me);
            let opcount = board.count(op);
            let ord = mycount.cmp(&opcount);
            rmin = ord;
            rmax = ord;
        } else {
            // Passで探索
            let mut board2 = board.clone();
            board2.apply_move(Move::Pass).unwrap();
            let t = search(&mut board2, mycolor, true);
            rmin = t.min;
            rmax = t.max;
            moves.push((Move::Pass, t));
        }
    }
    return GameTree {
        max: rmax,
        min: rmin,
        moves: Box::new(moves),
    };
}
