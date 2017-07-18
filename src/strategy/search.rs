// Search by alphabeta
use board::{Board, Tile};

// 評価
pub struct Evaluator {

}

impl Evaluator {
    fn new() -> Evaluator {
        Evaluator {}
    }
    // てきとうな評価関数
    // Blackに対してアレする
    fn evaluate(&self, board: &Board) -> i32 {
        let mut result = 0;
        // 石ごとの評価
        for x in 0..8 {
            for y in 0..8 {
                let t = board.get(x, y);
                if t == Tile::Black {
                    result += self.eval_place(x, y);
                } else if t == Tile::White {
                    result -= self.eval_place(x, y);
                }
            }
        }
        // 確定石の評価
        result += self.eval_fixed(board);
        result
    }
    fn eval_place(x: u8, y: u8) -> i32 {
        // 場所の評価
        // 参考: http://uguisu.skr.jp/othello/5-1.html
        lazy_static! {
            static ref PVALUE: Vec<i32> = vec![
                30 , -12,  0, -1, -1,  0, -12,  30,
                -12, -15, -3, -3, -3, -3, -15, -12,
                  0,  -3,  0, -1, -1,  0,  -3,   0,
                 -1,  -3, -1, -1, -1, -1,  -3,  -1,
                 -1,  -3, -1, -1, -1, -1,  -3,  -1,
                  0,  -3,  0, -1, -1,  0,  -3,   0,
                -12, -15, -3, -3, -3, -3, -15, -12,
                30 , -12,  0, -1, -1,  0, -12,  30,
            ];
        }
        let idx = ((y as usize) << 3) | (x as usize);
        PVALUE[idx]
    }
    fn eval_fixed(&self, board: &Board) -> i32 {
        // 係数はてきとう
        return 4 * fixed_check(board, Tile::Black) - 4 * fixed_check(board, Tile::White);
    }
}

// u64の上の表現
fn idx(x: u8, y: u8) -> u64 {
    1 << ((x as u64) | ((y as u64) << 3))
}

// 色ごとに
fn fixed_check(board: &Board, color: Tile) -> i32 {
    let mut fixed = 0u64; // ...321076543210

    // 外周
    // y=0
    let mut x = 0;
    while x < 8 {
        let t = board.get(x, 0);
        if t == color {
            fixed |= idx(x, 0);
        } else {
            break;
        }
        x += 1;
    }
    let mut xx = 7;
    while x < xx {
        let t = board.get(xx, 0);
        if t == color {
            fixed |= idx(xx, 0);
        } else {
            break;
        }
        xx -= 1;
    }
    // y = 7
    let mut x = 0;
    while x < 8 {
        let t = board.get(x, 7);
        if t == color {
            fixed |= idx(x, 7);
        } else {
            break;
        }
        x += 1;
    }
    let mut xx = 7;
    while x < xx {
        let t = board.get(xx, 0);
        if t == color {
            fixed |= idx(xx, 0);
        } else {
            break;
        }
        xx -= 1;
    }
    // 内側
    let mut changed = true;
    lazy_static! {
        static ref DVEC: Vec<(i32, i32)> = vec![
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
        ];
    }
    while changed {
        changed = false;
        for x in 1..7 {
            for y in 1..7 {
                if fixed & idx(x, y) == 0 && board.get(x, y) == color {
                    // 確定石チェックを走らせる
                    let mut ok = true;
                    'dloop: for &(dx, dy) in DVEC.iter() {
                        // 各方向について
                        let i1 = idx(x + dx, y + dy);
                        let i2 = idx(x - dx, y - dy);
                        if fixed & (i1 | i2) != 0 {
                            // 片方が確定石である: OK
                            continue;
                        }
                        // 両方向にfilled-rowチェックを走らせる
                        let mut xx = (x as i32) + dx;
                        let mut yy = (y as i32) + dy;
                        while 0 <= xx && xx <= 7 && 0 <= yy && yy <= 7 {
                            if board.get(xx as u8, yy as u8) == Tile::Empty {
                                // だめ
                                ok = false;
                                break 'dloop;
                            }
                            xx += dx;
                            yy += dy;
                        }
                        xx = (x as i32) - dx;
                        yy = (y as i32) - dy;
                        while 0 <= xx && xx <= 7 && 0 <= yy && yy <= 7 {
                            if board.get(xx as u8, yy as u8) == Tile::Empty {
                                // だめ
                                ok = false;
                                break 'dloop;
                            }
                            xx -= dx;
                            yy -= dy;
                        }
                        // filled-rowチェックを生き残った
                    }
                    if ok {
                        // これは確定石だ
                        field |= idx(x, y);
                        changed = true;
                    }
                }
            }
        }
    }
    // 結果を返す
    return fixed.count_ones() as i32;
    /*
    let mut st = 1;
    while st < 4 {
        // 四隅の判定
        let upleft = board.get(st, st) == color &&
            fixed & (1 << idx(st - 1, st)) != 0 &&
            fixed & (1 << idx(st, st - 1)) != 0;
        let upright = board.get(7 - st, st) == color &&
            fixed & (1 << idx(8 - st, st)) != 0 &&
            fixed & (1 << idx(7 - st, st - 1)) != 0;
        if upleft {
            // 左上から右へ
            if fixed & (1 << idx(st + 1, st - 1)) != 0 {
                // ここは確定石だ
                fixed |= 1 << idx(st, st);
                // 右に進む
                let mut xx = st + 1;
                while xx < 8 - st {
                    if board.get(xx, st) != color {
                        break;
                    }
                    if fixed & (1 << idx(xx + 1, st - 1)) == 0 {
                        break;
                    }
                    fixed |= 1 << idx(xx, st);
                }
            }
        }
    }
    */
}

pub struct Searcher {
    evaluator: Evaluator,
}
impl Searcher {
    fn new() -> Searcher {
        let evaluator = Evaluator::new();
        Searcher {
            evaluator,
        }
    }
}
