// Utilities for implementating strategies.
use std::iter::Iterator;

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
pub struct MoveIter {
    bits: u64,
    iter: usize,
}
impl MoveIter {
    fn new(board: &Board, turn: Turn) -> MoveIter {
        let (black, white) = board.bitboard();
        let (taker, taken) =
            if turn == Turn::Black {
                (black, white)
            } else {
                (white, black)
            };
        let mut takable = 0u64;
        // 8方向に

        // 左へ
        let area = taken & 0x7e7e7e7e7e7e7e7e;
        // 取る色の左隣にある取られる色（両端除く）
        let mut t = area & (taker >> 1);
        for _ in 0..5 {
            // さらにその左隣も入れる
            t |= area & (t >> 1);
        }
        takable |= t;
        // 上
        let area = taken & 0x00ffffffffffff00;
        let mut t = area & (taker >> 8);
        for _ in 0..5 {
            t |= area & (t >> 8);
        }
        takable |= t;
        // 右
        let area = taken & 0x7e7e7e7e7e7e7e7e;
        // 取る色の左隣にある取られる色（両端除く）
        let mut t = area & (taker << 1);
        for _ in 0..5 {
            // さらにその左隣も入れる
            t |= area & (t << 1);
        }
        takable |= t << 1;
        // 下
        let area = taken & 0x00ffffffffffff00;
        let mut t = area & (taker << 8);
        for _ in 0..5 {
            t |= area & (t << 8);
        }
        takable |= t << 8;
        // 左上
        let area = taken & 0x007e7e7e7e7e7e00;
        let mut t = area & (taker >> 9);
        for _ in 0..5 {
            t |= area & (t >> 9);
        }
        takable |= t >> 9;
        // 右上
        let area = taken & 0x007e7e7e7e7e7e00;
        let mut t = area & (taker >> 7);
        for _ in 0..5 {
            t |= area & (t >> 7);
        }
        takable |= t >> 7;
        // 左下
        let area = taken & 0x007e7e7e7e7e7e00;
        let mut t = area & (taker << 7);
        for _ in 0..5 {
            t |= area & (t << 7);
        }
        takable |= t << 7;
        // 右下
        let area = taken & 0x007e7e7e7e7e7e00;
        let mut t = area & (taker << 9);
        for _ in 0..5 {
            t |= area & (t << 9);
        }
        takable |= t << 9;

        // 石がないところに制限
        takable &= !(black | white);

        MoveIter {
            bits: takable,
            iter: 0,
        }
    }
    pub fn count_moves(&self) -> u32 {
        self.bits.count_ones()
    }
}
impl Iterator for MoveIter {
    type Item = Move;
    fn next(&mut self) -> Option<Move> {
        while self.iter < 64 {
            let i = self.iter;
            let idx = 1 << i;
            self.iter += 1;
            if self.bits & idx != 0 {
                let x = (i as u8) & 0x07;
                let y = (i as u8) >> 3;
                // x, yにおけるか?
                return Some(Move::Put {
                    x,
                    y,
                });
            }
        }
        return None;
    }
}

pub fn iter_moves(board: &Board, turn: Turn) -> MoveIter {
    MoveIter::new(board, turn)
}
