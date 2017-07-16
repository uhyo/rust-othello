// Othello board.
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

#[derive(Copy, Clone, Debug)]
pub enum Move{
    Pass,
    Put {
        x: u32,
        y: u32,
    },
}

pub trait Board: Debug{
    fn get(&self, x: u32, y: u32) -> Tile;
    fn set(&mut self, x: u32, y: u32, tile: Tile);

    fn get_turn(&self) -> Turn;
    fn set_turn(&mut self, turn: Turn);

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
                    (0, 0),
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
                        let t = self.get(cx as u32, cy as u32);
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
                                self.set(cx as u32, cy as u32, me);
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
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct VecBoard {
    turn: Turn,
    board: Vec<Tile>,
}

impl VecBoard {
    pub fn new() -> Self{
        let mut board = Vec::new();
        for i in 0 .. 63 {
            if i == 27 || i == 35 {
                board.push(Tile::White);
            } else if i == 28 || i == 34 {
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
    fn get(&self, x: u32, y: u32) -> Tile{
        let pos = (y * 8 + x) as usize;
        self.board[pos]
    }
    fn set(&mut self, x: u32, y: u32, tile: Tile){
        let pos = (y * 8 + x) as usize;
        self.board[pos] = tile;
    }

    fn get_turn(&self) -> Turn{
        self.turn
    }
    fn set_turn(&mut self, turn: Turn){
        self.turn = turn;
    }
}


/*
struct BitBoard {
    // left to right, then up to down
    black: u64,
    white: u64,
}
impl BitBoard {
    pub fn new() -> Self{
        let black = 0x00_00_00_08_10_00_00_00u64;
        let white = 0x00_00_00_10_08_00_00_00u64;
        
        BitBoard {
            black,
            white,
        }
    }
}
*/

pub fn make_board() -> VecBoard {
    VecBoard::new()
}

fn flip_turn(turn: Turn) -> Turn{
    match turn {
        Turn::Black => Turn::White,
        Turn::White => Turn::Black,
    }
}

// 自分のと相手の
fn turn_to_tile(turn: Turn) -> (Tile, Tile){
    match turn {
        Turn::Black => (Tile::Black, Tile::White),
        Turn::White => (Tile::White, Tile::Black),
    }
}
