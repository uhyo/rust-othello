// Othello board.
use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub enum Tile{
    Empty,
    Black,
    White,
}

#[derive(Copy, Clone, Debug)]
pub enum Turn{
    Black,
    White,
}

pub trait Board: Debug{
    fn get(&self, x: u32, y: u32) -> Tile;
    fn set(&mut self, x: u32, y: u32, tile: Tile);

    fn get_turn(&self) -> Turn;
    fn set_turn(&mut self, turn: Turn);
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
