// Utilities for implementating strategies.
use std::iter::Iterator;
use std::ops::Range;

use board::{Board, Tile, Turn, Move, turn_to_tile};

// この場所に石を置けるか?
pub fn putable(board: &Board, x: u8, y: u8, turn: Turn) -> bool {
    if board.get(x, y) != Tile::Empty {
        // 置けない
        return false;
    }
    let (me, op) = turn_to_tile(turn);
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

// iterは左上から順番に返すような感じがする
pub struct MoveIter<'a> {
    board: &'a Board,
    turn: Turn,
    iter: Range<u8>,
}
impl<'a> MoveIter<'a> {
    fn new(board: &'a Board, turn: Turn) -> MoveIter<'a> {
        let iter = 0..64;
        MoveIter {
            board,
            turn,
            iter,
        }
    }
}
impl<'a> Iterator for MoveIter<'a> {
    type Item = Move;
    fn next(&mut self) -> Option<Move> {
        while let Some(i) = self.iter.next() {
            let x = i >> 3;
            let y = i & 0x7;
            // x, yにおけるか?
            if putable(self.board, x, y, self.turn) {
                return Some(Move::Put {
                    x,
                    y,
                });
            }
        }
        return None;
    }
}

pub fn iter_moves<'a>(board: &'a Board, turn: Turn) -> MoveIter<'a> {
    MoveIter::new(board, turn)
}
