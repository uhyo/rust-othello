// Utilities for implementating strategies.
use board::{Board, Tile, turn_to_tile};

// この場所に石を置けるか?
pub fn putable(board: &Board, x: u8, y: u8) -> bool{
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

