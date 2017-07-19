// Search by alphabeta

use board::{Board, Tile, Move, Turn, flip_turn};
use strategy::util::iter_moves;

// 評価
pub struct Evaluator {
    fixed_cache_black: u64,
    fixed_cache_white: u64,
}

impl Evaluator {
    fn new() -> Evaluator {
        Evaluator {
            fixed_cache_black: 0,
            fixed_cache_white: 0,
        }
    }
    fn reset(&mut self) {
        self.fixed_cache_black = 0;
        self.fixed_cache_white = 0;
    }
    // てきとうな評価関数
    // Blackに対してアレする
    fn evaluate(&mut self, board: &Board) -> i32 {
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
        result += self.eval_stable(board);
        result
    }
    fn eval_place(&self, x: u8, y: u8) -> i32 {
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
    fn eval_stable(&mut self, board: &Board) -> i32 {
        // 係数はてきとう
        let (fb2, bc) = stable_check(board, Tile::Black, self.fixed_cache_black);
        let (fw2, wc) = stable_check(board, Tile::White, self.fixed_cache_white);
        self.fixed_cache_black = fb2;
        self.fixed_cache_white = fw2;
        4 * (bc as i32) - 4 * (wc as i32)
    }
}

// u64の上の表現
fn idx(x: u8, y: u8) -> u64 {
    1 << ((x as u64) | ((y as u64) << 3))
}

// 色ごとに
pub fn stable_check(board: &Board, color: Tile, fixedcache: u64) -> (u64, u32) {
    let mut fixed = fixedcache; // ...321076543210

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
        let t = board.get(xx, 7);
        if t == color {
            fixed |= idx(xx, 7);
        } else {
            break;
        }
        xx -= 1;
    }
    // 縦 x = 0
    let mut y = 0;
    while y < 7 {
        let t = board.get(0, y);
        if t == color {
            fixed |= idx(0, y);
        } else {
            break;
        }
        y += 1;
    }
    let mut yy = 7;
    while y < yy {
        let t = board.get(0, yy);
        if t == color {
            fixed |= idx(0, yy);
        } else {
            break;
        }
        yy -= 1;
    }
    // x = 7
    y = 0;
    while y < 7 {
        let t = board.get(7, y);
        if t == color {
            fixed |= idx(7, y);
        } else {
            break;
        }
        y += 1;
    }
    yy = 7;
    while y < yy {
        let t = board.get(7, yy);
        if t == color {
            fixed |= idx(7, yy);
        } else {
            break;
        }
        yy -= 1;
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
        for x in 0..8 {
            'ploop: for y in 0..8 {
                let xi32 = x as i32;
                let yi32 = y as i32;
                if fixed & idx(x, y) == 0 && board.get(x, y) == color {
                    // 確定石チェックを走らせる
                    'dloop: for &(dx, dy) in DVEC.iter() {
                        // 各方向について
                        let xdi = xi32 + dx;
                        let ydi = yi32 + dy;
                        let xdi2 = xi32 - dx;
                        let ydi2 = yi32 - dy;
                        let i1 =
                            if 0 <= xdi && xdi <= 7 && 0 <= ydi && ydi <= 7 {
                                idx(xdi as u8, ydi as u8)
                            }else{
                                0xffffffff
                            };
                        let i2 =
                            if 0 <= xdi2 && xdi2 <= 7 && 0 <= ydi2 && ydi2 <= 7 {
                                idx(xdi2 as u8, ydi2 as u8)
                            }else{
                                0xffffffff
                            };
                        if fixed & (i1 | i2) != 0 {
                            // 片方が確定石である: OK
                            continue;
                        }
                        // 両方向にfilled-rowチェックを走らせる
                        let mut xx = xi32 + dx;
                        let mut yy = yi32 + dy;
                        while 0 <= xx && xx <= 7 && 0 <= yy && yy <= 7 {
                            if board.get(xx as u8, yy as u8) == Tile::Empty {
                                // だめ
                                continue 'ploop;
                            }
                            xx += dx;
                            yy += dy;
                        }
                        xx = xi32 - dx;
                        yy = yi32 - dy;
                        while 0 <= xx && xx <= 7 && 0 <= yy && yy <= 7 {
                            if board.get(xx as u8, yy as u8) == Tile::Empty {
                                // だめ
                                continue 'ploop;
                            }
                            xx -= dx;
                            yy -= dy;
                        }
                        // filled-rowチェックを生き残った
                    }
                    // これは確定石だ
                    // println!("({}, {}) is stable!", x, y);
                    fixed |= idx(x, y);
                    changed = true;
                }
            }
        }
    }
    // 結果を返す
    return (fixed, fixed.count_ones());
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
    pub fn new() -> Searcher {
        let evaluator = Evaluator::new();
        Searcher {
            evaluator,
        }
    }
    pub fn search<B>(&mut self, board: &B) -> Move
        where B: Board + Clone {
        // てきとう
        let depth = 6;
        let mycolor = board.get_turn();
        let (_, mv) = alphabeta(&mut self.evaluator, board, mycolor, depth, <i32>::min_value(), <i32>::max_value(), None);
        mv
    }
    pub fn reset(&mut self) {
        self.evaluator.reset();
    }
}

fn alphabeta<B>(evaluator: &mut Evaluator, board: &B, mycolor: Turn, depth: u32, alpha: i32, beta: i32, mv: Option<Move>) -> (i32, Move) where B: Board + Clone {
    if depth == 0 {
        return (evaluator.evaluate(board), mv.unwrap_or(Move::Pass));
    }
    // 石をおける場所を列挙
    let mut alpha = alpha;
    let mut beta = beta;
    if board.get_turn() == mycolor {
        let mut flg = false;
        let mut result_mv = mv.unwrap_or(Move::Pass);
        for mv2 in iter_moves(board) {
            flg = true;
            let nmv = mv.or(Some(mv2));
            // 次の盤面を生成
            let mut board2 = board.clone();
            board2.apply_move(mv2).unwrap();

            let (av, mv3) = alphabeta(evaluator, &board2, flip_turn(mycolor), depth-1, alpha, beta, nmv);
            // 常に黒の評価値なので白だったら逆に
            let av = if mycolor == Turn::Black {
                av 
            } else {
                -av
            };
            if alpha <= av {
                alpha = av;
                result_mv = mv3;
            }
            if alpha >= beta {
                // cut
                return (beta, mv3);
            }
        }
        if flg {
            return (alpha, result_mv);
        }
    } else {
        let mut flg = false;
        let mut result_mv = mv.unwrap_or(Move::Pass);
        for mv2 in iter_moves(board) {
            flg = true;
            let nmv = mv.or(Some(mv2));
            let mut board2 = board.clone();
            board2.apply_move(mv2).unwrap();

            let (bv, mv3) = alphabeta(evaluator, &board2, flip_turn(mycolor), depth-1, alpha, beta, nmv);
            // 常に黒の評価値なので白だったら逆に
            let bv = if mycolor == Turn::Black {
                bv 
            } else {
                -bv
            };
            if bv <= beta {
                beta = bv;
                result_mv = mv3;
            }
            if alpha >= beta {
                // cut
                return (alpha, mv3);
            }
        }
        if flg {
            return (beta, result_mv);
        }
    }
    // 子とかなかった
    return (evaluator.evaluate(board), mv.unwrap_or(Move::Pass));
}

