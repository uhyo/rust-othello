// Seach for ending
use std::cmp::Ordering;
use board::{Board, Tile, Move, Turn, turn_to_tile};
use strategy::util::{putable, MoveIter, iter_moves};

pub struct EndingSearcher {
}
impl EndingSearcher {
    fn new() -> EndingSearcher {
        EndingSearcher {
        }
    }
    fn search<B>(&self, board: &B, mycolor: Turn) -> Move where B: Board {
        // ゲームの終わりまでサーチする
        search(board, mycolor)
    }
}

fn search<B>(board: &B, mycolor: Turn, mv: Option<Move>) -> (Ordering, Move) where B: Board {
    for mv in iter_moves(board) {

    }
    // 終局っぽいので数を数える
    let (me, op) = turn_to_tile(mycolor);
    let mycount = board.count(me);
    let opcount = board.count(op);
    return (mycount.cmp(opcount), mv.unwrap_or(Move::Pass));
}
