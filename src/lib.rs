#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate regex;
extern crate getopts;
extern crate rand;
extern crate byteorder;

pub mod options;
pub mod client;
pub mod board;
pub mod strategy;
pub mod matcher;

#[cfg(test)]
mod test {

    mod stable_check {
        use strategy::search::stable_check;
        use board::{Board, VecBoard, BitBoard, Tile};

        #[test]
        fn edges_up() {
            let mut board = VecBoard::new();
            // boardを初期化
            board.set(0, 0, Tile::Black);
            board.set(1, 0, Tile::Black);
            board.set(2, 0, Tile::Black);
            board.set(6, 0, Tile::Black);
            board.set(7, 0, Tile::Black);
            assert_eq!(stable_check(&board, Tile::Black, 0).1, 5);
        }
        #[test]
        fn edges_updown() {
            let mut board = BitBoard::new();
            // boardを初期化
            board.set(0, 0, Tile::Black);
            board.set(1, 0, Tile::Black);
            board.set(2, 0, Tile::Black);
            board.set(6, 0, Tile::Black);
            board.set(7, 0, Tile::Black);

            board.set(0, 7, Tile::Black);
            board.set(1, 7, Tile::Black);

            board.set(3, 7, Tile::Black);

            board.set(5, 7, Tile::Black);
            board.set(6, 7, Tile::Black);
            board.set(7, 7, Tile::Black);
            assert_eq!(stable_check(&board, Tile::Black, 0).1, 10);
        }
        #[test]
        fn edges_leftright() {
            let mut board = VecBoard::new();
            // boardを初期化
            board.set(0, 0, Tile::Black);
            board.set(0, 1, Tile::Black);
            board.set(0, 2, Tile::Black);
            board.set(0, 3, Tile::Black);
            board.set(0, 6, Tile::Black);
            board.set(0, 7, Tile::Black);

            board.set(7, 0, Tile::Black);
            board.set(7, 1, Tile::Black);
            board.set(7, 2, Tile::Black);
            board.set(7, 3, Tile::Black);

            assert_eq!(stable_check(&board, Tile::Black, 0).1, 10);
        }
        #[test]
        fn inner_stones() {
            let b = Tile::Black;
            let w = Tile::White;
            let e = Tile::Empty;
            let board = make_board(vec![
                b, b, b, b, w, b, b, b,
                b, b, b, e, w, b, b, b,
                e, e, e, w, w, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
            ]);

            assert_eq!(stable_check(&board, Tile::Black, 0).1, 12);
        }
        #[test]
        fn fulfilled_stones() {
            let b = Tile::Black;
            let w = Tile::White;
            let e = Tile::Empty;
            let board = make_board(vec![
                b, b, b, b, e, w, b, b,
                b, b, b, e, e, e, b, w,
                b, b, e, e, e, e, e, b,
                w, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
            ]);

            assert_eq!(stable_check(&board, Tile::Black, 0).1, 11);
        }
        #[test]
        fn edge_fulfilled() {
            let b = Tile::Black;
            let w = Tile::White;
            let e = Tile::Empty;
            let board = make_board(vec![
                b, b, w, w, b, w, w, b,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
                e, e, e, e, e, e, e, e,
            ]);

            assert_eq!(stable_check(&board, Tile::Black, 0).1, 4);
        }
        /*
        #[test]
        fn other() {
            let b = Tile::Black;
            let w = Tile::White;
            let e = Tile::Empty;
            let board = make_board(vec![
                b, b, b, w, w, w, w, w,
                b, b, b, b, w, b, w, b,
                b, b, b, w, b, w, b, b,
                w, b, b, b, w, b, b, b,
                e, w, b, b, b, b, b, b,
                w, w, w, b, b, b, b, b,
                e, w, b, b, b, b, b, b,
                w, w, w, w, w, w, w, b,
            ]);

            println!("{:b}",stable_check(&board, Tile::Black, 0).0);
            assert_eq!(stable_check(&board, Tile::Black, 0), (0b10000000_11111100_11111000_11111100_11101100_11010011_10100111_00000111, 36));
        }
        */



        fn make_board(v: Vec<Tile>) -> BitBoard {
            // vecからBoardをつくる
            let mut board = BitBoard::new();
            for x in 0..8 {
                for y in 0..8 {
                    let idx = y * 8 + x;
                    board.set(x, y, v[idx as usize]);
                }
            }
            board
        }
    }
    
}
