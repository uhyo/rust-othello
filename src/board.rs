// Othello board.
use std::fmt;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tile{
    Empty,
    Black,
    White,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Turn{
    Black,
    White,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Move{
    Pass,
    Put {
        x: u8,
        y: u8,
    },
}
impl fmt::Display for Move{
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Move::Pass => write!(f, "Pass"),
            Move::Put {x, y} => write!(f, "Put ({}, {})", x, y),
        }
    }
}

pub trait Board: Debug {
    fn reset(&mut self);
    fn get(&self, x: u8, y: u8) -> Tile;
    fn set(&mut self, x: u8, y: u8, tile: Tile);

    fn get_turn(&self) -> Turn;
    fn set_turn(&mut self, turn: Turn);

    // count pieces.
    fn count(&self, tile: Tile) -> u32 {
        let mut result = 0;
        for x in 0..8 {
            for y in 0..8 {
                if self.get(x, y) == tile {
                    result += 1;
                }
            }
        }
        result
    }
    fn count_both(&self) -> u32 {
        let mut result = 0;
        for x in 0..8 {
            for y in 0..8 {
                if self.get(x, y) != Tile::Empty {
                    result += 1;
                }
            }
        }
        result
    }

    // bitboard data
    // VecBoardの存在意義 is 何
    fn bitboard(&self) -> (u64, u64);

    // apply a move to board.
    fn apply_move(&mut self, mv: Move) -> Result<(), String>{
        match mv {
            Move::Pass => {
                // turnだけ変える
                let t = self.get_turn();
                self.set_turn(flip_turn(t));
                Ok(())
            },
            Move::Put {x, y} => {
                let (me, op) = turn_to_tile(self.get_turn());
                
                if self.get(x, y) != Tile::Empty {
                    return Err(String::from("Tile already exists"));
                }
                // 8方向に探索
                let dirs: Vec<(i32, i32)> = vec![
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ];
                let mut take_flg = false;
                for (dx, dy) in dirs {
                    let mut flag: u8 = 0;
                    let mut cx = x as i32;
                    let mut cy = y as i32;
                    loop {
                        cx += dx;
                        cy += dy;
                        if cx < 0 || cx > 7 || cy < 0 || cy > 7 {
                            break;
                        }
                        let t = self.get(cx as u8, cy as u8);
                        if flag == 0 {
                            if t != op {
                                // 取れないわ
                                break;
                            }
                            flag = 1;
                        } else if flag == 1 {
                            if t == Tile::Empty {
                                // 取れなかった
                                break;
                            }
                            if t == me {
                                // 取れるわ
                                flag = 2;
                                cx = x as i32;
                                cy = y as i32;
                                continue;
                            }
                        } else if flag == 2 {
                            // 取ってる
                            if t == op {
                                self.set(cx as u8, cy as u8, me);
                                take_flg = true;
                            } else {
                                break;
                            }
                        }
                    }
                }
                if !take_flg {
                    // 石とれてない
                    return Err(String::from("Cannot take pieces"));
                }
                // できたから諸々の処理
                self.set(x, y, me);
                let t = self.get_turn();
                self.set_turn(flip_turn(t));
                Ok(())
            }
        }
    }
    // Pretty print the board.
    fn pretty_print(&self) -> String{
        let mut result = String::new();
        result.push_str("  A B C D E F G H\n");
        for y in 0..8 {
            result.push(char::from(0x31 + y));
            result.push(' ');
            for x in 0..8 {
                let t = self.get(x, y);
                match t {
                    Tile::Empty => result.push(' '),
                    Tile::Black => result.push('x'),
                    Tile::White => result.push('o'),
                };
                if x < 7 {
                    result.push('|');
                }
            }
            if y < 7 {
                result.push_str("\n --+-+-+-+-+-+-+-\n");
            } else {
                result.push('\n');
            }
        }
        return result;
    }
}

#[derive(Debug, Clone)]
pub struct VecBoard {
    turn: Turn,
    board: Vec<Tile>,
}

impl VecBoard {
    pub fn new() -> Self{
        let mut board = Vec::new();
        for i in 0 .. 64 {
            if i == 27 || i == 36 {
                board.push(Tile::White);
            } else if i == 28 || i == 35 {
                board.push(Tile::Black);
            } else {
                board.push(Tile::Empty);
            }
        }

        VecBoard {
            turn: Turn::Black,
            board,
        }
    }
}
impl Board for VecBoard{
    fn reset(&mut self) {
        self.board.clear();
        for i in 0 .. 64 {
            if i == 27 || i == 36 {
                self.board.push(Tile::White);
            } else if i == 28 || i == 35 {
                self.board.push(Tile::Black);
            } else {
                self.board.push(Tile::Empty);
            }
        }
        self.turn = Turn::Black;
    }
    fn get(&self, x: u8, y: u8) -> Tile{
        let pos = (y * 8 + x) as usize;
        self.board[pos]
    }
    fn set(&mut self, x: u8, y: u8, tile: Tile){
        let pos = (y * 8 + x) as usize;
        self.board[pos] = tile;
    }

    fn get_turn(&self) -> Turn{
        self.turn
    }
    fn set_turn(&mut self, turn: Turn){
        self.turn = turn;
    }
    fn bitboard(&self) -> (u64, u64) {
        let mut i = 1u64;
        let mut black = 0u64;
        let mut white = 0u64;
        for y in 0..8 {
            for x in 0..8 {
                match self.get(x, y) {
                    Tile::Black => {
                        black |= i;
                    },
                    Tile::White => {
                        white |= i;
                    },
                    Tile::Empty => (),
                }
                i += 1;
            }
        }
        return (black, white);
    }
}


#[derive(Debug, Clone)]
pub struct BitBoard {
    turn: Turn,
    black: u64,
    white: u64,
}
impl BitBoard {
    pub fn new() -> Self{
        // 左下 to 右上
        let black = 0x00_00_00_08_10_00_00_00u64;
        let white = 0x00_00_00_10_08_00_00_00u64;
        
        BitBoard {
            turn: Turn::Black,
            black,
            white,
        }
    }
}
impl Board for BitBoard {
    fn reset(&mut self) {
        self.turn = Turn::Black;
        self.black = 0x00_00_00_08_10_00_00_00u64;
        self.white = 0x00_00_00_10_08_00_00_00u64;
    }
    fn get(&self, x: u8, y: u8) -> Tile {
        let i = idx(x, y);
        if self.black & i != 0 {
            return Tile::Black;
        } else if self.white & i != 0 {
            return Tile::White;
        } else {
            return Tile::Empty;
        }
    }
    fn set(&mut self, x: u8, y: u8, tile: Tile) {
        let i = idx(x, y);
        match tile {
            Tile::Black => {
                self.black |= i;
                self.white &= !i;
            },
            Tile::White => {
                self.white |= i;
                self.black &= !i;
            },
            Tile::Empty => {
                self.black &= !i;
                self.white &= !i;
            },
        }
    }
    fn get_turn(&self) -> Turn {
        self.turn
    }
    fn set_turn(&mut self, turn: Turn) {
        self.turn = turn;
    }
    fn bitboard(&self) -> (u64, u64) {
        return (self.black, self.white);
    }
    fn count(&self, tile: Tile) -> u32 {
        match tile {
            Tile::Black => self.black.count_ones(),
            Tile::White => self.white.count_ones(),
            Tile::Empty => (self.black | self.white).count_zeros(),
        }
    }
    fn count_both(&self) -> u32 {
        (self.black | self.white).count_ones()
    }
    fn apply_move(&mut self, mv: Move) -> Result<(), String> {
        match mv {
            Move::Pass => {
                let t = self.get_turn();
                self.set_turn(flip_turn(t));
                Ok(())
            },
            Move::Put {x, y} => {
                let (taker, taken) =
                    if self.get_turn() == Turn::Black {
                        (self.black, self.white)
                    } else {
                        (self.white, self.black)
                    };

                let mut result = 0u64;
                let mv = idx(x, y);
                if taker & mv != 0 {
                    return Err(String::from("Tile already exists"));
                }
                // 左
                let mut r = 0u64;
                let mask = 0x7f7f7f7f7f7f7f7f;
                let mut moo = (mv >> 1) & mask;
                while moo & taken != 0 {
                    // こちらの方向に取れる石が続いている
                    r |= moo;
                    moo = (moo >> 1) & mask;
                }
                if moo & taker != 0 {
                    //終点にも石があったので取れる
                    result |= r;
                }
                // 上
                let mut r = 0u64;
                let mask = 0x00ffffffffffffff;
                let mut moo = (mv >> 8) & mask;
                while moo & taken != 0 {
                    r |= moo;
                    moo = (moo >> 8) & mask;
                }
                if moo & taker != 0 {
                    result |= r;
                }
                // 右
                let mut r = 0u64;
                let mask = 0xfefefefefefefefe;
                let mut moo = (mv << 1) & mask;
                while moo & taken != 0 {
                    // こちらの方向に取れる石が続いている
                    r |= moo;
                    moo = (moo << 1) & mask;
                }
                if moo & taker != 0 {
                    //終点にも石があったので取れる
                    result |= r;
                }
                // 下
                let mut r = 0u64;
                let mask = 0xffffffffffffff00;
                let mut moo = (mv << 8) & mask;
                while moo & taken != 0 {
                    r |= moo;
                    moo = (moo << 8) & mask;
                }
                if moo & taker != 0 {
                    result |= r;
                }
                // 左上
                let mut r = 0u64;
                let mask = 0x007f7f7f7f7f7f7f;
                let mut moo = (mv >> 9) & mask;
                while moo & taken != 0 {
                    r |= moo;
                    moo = (moo >> 9) & mask;
                }
                if moo & taker != 0 {
                    result |= r;
                }
                // 右上
                let mut r = 0u64;
                let mask = 0x00fefefefefefefe;
                let mut moo = (mv >> 7) & mask;
                while moo & taken != 0 {
                    r |= moo;
                    moo = (moo >> 7) & mask;
                }
                if moo & taker != 0 {
                    result |= r;
                }
                // 左下
                let mut r = 0u64;
                let mask = 0x7f7f7f7f7f7f7f00;
                let mut moo = (mv << 7) & mask;
                while moo & taken != 0 {
                    r |= moo;
                    moo = (moo << 7) & mask;
                }
                if moo & taker != 0 {
                    result |= r;
                }
                // 右下
                let mut r = 0u64;
                let mask = 0xfefefefefefefe00;
                let mut moo = (mv << 9) & mask;
                while moo & taken != 0 {
                    r |= moo;
                    moo = (moo << 9) & mask;
                }
                if moo & taker != 0 {
                    result |= r;
                }
                // 終了
                if result == 0 {
                    trace!("ah... \n {}", self.pretty_print());
                    return Err(format!("Cannot take pieces {:?} {}, {}", self.get_turn(), x, y));
                }
                let t = self.get_turn();
                if t == Turn::Black {
                    self.black |= mv | result;
                    self.white ^= result;
                } else {
                    self.white |= mv | result;
                    self.black ^= result;
                }
                self.set_turn(flip_turn(t));
                return Ok(());
            },
        }
    }
}
fn idx(x: u8, y: u8) -> u64 {
    1 << ((x as u64) | ((y as u64) << 3))
}

pub fn make_board() -> BitBoard {
    BitBoard::new()
}

pub fn flip_turn(turn: Turn) -> Turn{
    match turn {
        Turn::Black => Turn::White,
        Turn::White => Turn::Black,
    }
}

// 自分のと相手の
pub fn turn_to_tile(turn: Turn) -> (Tile, Tile){
    match turn {
        Turn::Black => (Tile::Black, Tile::White),
        Turn::White => (Tile::White, Tile::Black),
    }
}
