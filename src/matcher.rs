// Do a match between two strategies.
use std::mem;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use rand;

use options::Opts;
use strategy::{make_learner, Strategy};

use board::{Board, Move, Tile, make_board};

struct MatchResult {
    moves: Vec<Move>,
    black: u32,
    white: u32,
}

// 1ゲーム行う
fn do_match<S>(strategy1: S, strategy2: S) -> Result<MatchResult, String>
    where S: Strategy {

    let mut board = make_board();

    let mut turnplayer = strategy1;
    let mut opponent   = strategy2;

    turnplayer.reset();
    opponent.reset();

    if rand::random() {
        mem::swap(&mut turnplayer, &mut opponent);
    }
    // double_passが起きるまで対局
    let mut last_move = None;
    let mut pass_count = 0;
    let mut moves = Vec::new();

    while pass_count < 2 {
        let mv = turnplayer.play(&board, last_move, 60000);
        if mv == Move::Pass {
            pass_count += 1;
        } else {
            pass_count = 0;
        }
        board.apply_move(mv)?;
        moves.push(mv);
        last_move = Some(mv);

        // ターンを交代
        mem::swap(&mut turnplayer, &mut opponent);
    }
    // 終局
    let black = board.count(Tile::Black);
    let white = board.count(Tile::White);

    Ok(MatchResult {
        moves,
        black,
        white,
    })
}

static DATANAME: &str = "data/record.db";

pub fn match_mode(opts: &Opts) -> io::Result<()>{
    // 結果をファイルに書き込み
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(DATANAME)?;

    let mut game_count = 0u32;
    loop {
        // 1ゲーム行う
        let strategy1 = make_learner(opts);
        let strategy2 = make_learner(opts);

        match do_match(strategy1, strategy2) {
            Err(err)=>{
                warn!("Error: {}", err);
            },
            Ok(res)=>{
                let mut buf: Vec<u8> = Vec::with_capacity(64);
                let mut i = 0;
                for mv in res.moves.iter() {
                    match *mv {
                        Move::Pass => {
                            // Pass
                            buf.push(0x88)
                        },
                        Move::Put {x, y} => {
                            buf.push((x << 4) | y);
                        },
                    }
                    i += 1;
                    if i == 60 {
                        // 最大60手のはず
                        break;
                    }
                }
                // 残りを埋める
                while i < 62 {
                    buf.push(0xff);
                    i += 1;
                }
                // 石の数の情報
                buf.push(res.black as u8);
                buf.push(res.white as u8);
                // write
                file.write_all(&buf)?;
                file.flush()?;

                info!("Write {} bytes", buf.len());
                game_count += 1;
                if game_count % 10 == 0 {
                    info!("{} games written", game_count);
                }
            },
        }
    }
}
