// Seach for ending
use std::cmp::{min, max, Ordering};
use options::Opts;
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
    pub fn take(mut self, mv: Move) -> Option<Self> {
        let mut idx = 0;
        let mut flag = false;
        for (i, &(mvv, _)) in self.moves.iter().enumerate() {
            if mv == mvv {
                // これだ
                flag = true;
                idx = i;
                break;
            }
        }
        if flag {
            // 指定された手があった
            return Some(self.moves.swap_remove(idx).1);
        } else {
            // 指定された手がなかった
            return None;
        }
    }
}

pub struct EndingSearcher {
    // 探索した結果の木
    table: Option<GameTree>,
    // 相手の手に最良を仮定して探索
    ending_opt: bool,
}
impl EndingSearcher {
    pub fn new(opts: &Opts) -> EndingSearcher {
        EndingSearcher {
            table: None,
            ending_opt: opts.ending_opt,
        }
    }
    pub fn reset(&mut self) {
        self.table = None;
    }
    pub fn search<B>(&mut self, board: &B, mycolor: Turn, last_move: Move) -> Move where B: Board + Clone {
        let t = self.table.take().and_then(|tt| tt.take(last_move));
        let table =
            match t {
                Some(tt) => {
                    tt
                },
                None => {
                    // ゲームの終わりまでサーチする
                    search(self.ending_opt, board, mycolor, false)
                }
            };
        match table.moves.first() {
            None => {
                // 動けない
                self.table = Some(table);
                return Move::Pass;
            },
            Some(&(mv, _)) => {
                self.table = table.take(mv);
                trace!("Choose {:?}/{:?} move", self.table.as_ref().unwrap().max, self.table.as_ref().unwrap().min);
                return mv;
            },
        }
    }
    // 勝手に手がひとつ進んだ
    pub fn go(&mut self, last_move: Move) {
        if self.table.is_none() {
            // まだ探索していないから無視
            return;
        }
        // tableの手を進める
        self.table = self.table.take().and_then(|tt| tt.take(last_move));
    }
}

fn search<B>(ending_opt: bool, board: &B, mycolor: Turn, one_pass: bool) -> GameTree where B: Board + Clone {
    let mut flg = false;
    let mut rmin = Ordering::Greater;
    let mut rmax = Ordering::Less;
    let mut moves = Vec::new();
    for mv in iter_moves(board, board.get_turn()) {
        flg = true;

        if let Move::Put {x: _, y: _} = mv {
            // 次の盤面
            let mut board2 = board.clone();
            board2.apply_move(mv).unwrap();
            let t = search(ending_opt, &mut board2, mycolor, false);
            rmin = min(rmin, t.min);
            rmax = max(rmax, t.max);
            if t.min == Ordering::Greater && board.get_turn() == mycolor {
                // いいのを見つけた（これでいいじゃん）
                moves.clear();
                rmin = Ordering::Greater;
                rmax = Ordering::Greater;
                moves.push((mv, t));
                break;
            } else if t.max == Ordering::Less && board.get_turn() != mycolor {
                // 相手の手がやばい
                moves.clear();
                rmin = Ordering::Less;
                rmax = Ordering::Less;
                moves.push((mv, t));
                break;
            } else {
                moves.push((mv, t));
            }
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
            let t = search(ending_opt, &mut board2, mycolor, true);
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
