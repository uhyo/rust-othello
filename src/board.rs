// Othello board.

#[derive(Copy, Clone)]
pub enum Tile{
    Empty,
    Black,
    White,
}

pub trait Board{
    fn get(&self, x: u32, y: u32) -> Tile;
}

pub struct VecBoard {
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
            board,
        }
    }
}
impl Board for VecBoard{
    fn get(&self, x: u32, y: u32) -> Tile{
        let pos = (y * 8 + x) as usize;
        self.board[pos]
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
