// ISOP (IS Othello Protocol) client.

use std::net::TcpStream;
use std::io;
use std::io::{Write, BufRead, BufReader};

use regex::Regex;

use options;
use board::{Board, Turn, Move};
use strategy::Strategy;

// コマンドのパースに使う正規表現たち
lazy_static! {
    // BYEコマンド
    static ref RBYE: Regex = Regex::new(r"^BYE(?:$|\s)").unwrap();
    // STARTコマンド
    static ref RSTART: Regex = Regex::new(r"^START (BLACK|WHITE) (\S+) (\d+)").unwrap();
    // ENDコマンド
    static ref REND: Regex = Regex::new(r"^END (Win|Lose|Tie) (\d+) (\d+)(.*)").unwrap();
    // MOVEコマンド
    static ref RMOVE: Regex = Regex::new(r"^MOVE (PASS|[A-H][1-8])$").unwrap();
}

#[derive(Debug)]
pub struct Client{
    name: String,
    stream: BufReader<TcpStream>,
    // my color
    color: Turn,
    // remaining time
    time: i32,
}

impl Client{
    pub fn new(opts: &options::Opts) -> io::Result<Self>{
        let name = opts.name.clone();
        let stream = TcpStream::connect((opts.host.as_ref(), opts.port))?;
        info!("Connected to the server");

        let stream = BufReader::new(stream);
        Ok(Client {
            name,
            stream,
            color: Turn::Black,
            time: 0,
        })
    }

    // perform init 
    pub fn run(&mut self, board: &mut Board, strategy: &mut Strategy) -> io::Result<()>{
        // first, we send the `NAME` command
        writeln!(self.stream.get_mut(), "OPEN {}", self.name)?;
        debug!("Registered as client {}", self.name);

        self.wait_for_game(board, strategy)
    }

    // 'waiting for game' state
    fn wait_for_game(&mut self, board: &mut Board, strategy: &mut Strategy) -> io::Result<()>{
        let mut buf = String::new();
        self.stream.read_line(&mut buf)?;
        if RBYE.is_match(&buf) {
            // BYEコマンドを受け取った
            debug!("{}", buf.trim());
            info!("Connection closed by the server");
            return Ok(());
        }
        if let Some(caps) = RSTART.captures(&buf) {
            debug!("{}", buf.trim());
            let color = caps.get(1).unwrap().as_str();
            let opponent = caps.get(2).map_or("UNKNOWN", |m| m.as_str());
            let time: i32 = caps.get(3).and_then(|m| m.as_str().parse().ok()).unwrap();
            info!("Game started - vs \"{}\"", opponent);
            // 情報をセット
            if color == "BLACK" {
                self.color = Turn::Black;
            } else {
                self.color = Turn::White;
            }
            self.time = time;

            return self.turn(board, strategy);
        }

        debug!("{}", buf.trim());
        warn!("Invalid command sent from the server");

        Ok(())
    }
    // 'turn' state
    fn turn(&mut self, board: &mut Board, strategy: &mut Strategy) -> io::Result<()>{
        if board.get_turn() == self.color {
            self.my_turn(board, strategy)
        } else {
            self.opponent_turn(board, strategy)
        }
    }
    // opponent's turn
    fn opponent_turn(&mut self, board: &mut Board, strategy: &mut Strategy) -> io::Result<()>{
        // 相手ターンなので通信を待機
        // TODO 相手ターンでも探索したらいいのでは?
        let mut buf = String::new();
        self.stream.read_line(&mut buf)?;

        if let Some(caps) = REND.captures(&buf) {
            // 結果表示
            let mut buf = String::new();
            caps.expand("Game ended: $1 ($2/$3) - $4", &mut buf);
            info!("{}", buf.trim());

            return Ok(());
        }
        if let Some(caps) = RMOVE.captures(&buf) {
            let mv = caps.get(1).unwrap().as_str();
            if let Some(mv) = parse_move(&mv) {
                // 動かす
                return match board.apply_move(mv) {
                    Ok(()) => {
                        // 次のターンへ
                        self.turn(board, strategy)
                    },
                    Err(s) => {
                        // おかしい着手が来たぞ
                        warn!("Invalid move sent from the server - {}", s);
                        Ok(())
                    },
                }
            }
        }
        // 変なレスポンス
        debug!("{}", buf.trim());
        warn!("Invalid command sent from the server");
        Ok(())
    }
    // my turn
    fn my_turn(&mut self, board: &mut Board, strategy: &mut Strategy) -> io::Result<()>{
        Ok(())
    }
}

// moveをparseする
fn parse_move(mv: &str) -> Option<Move>{
    if mv == "PASS" {
        Some(Move::Pass)
    } else {
        let mut chars = mv.chars();
        if let Some(c1) = chars.next() {
            if let Some(c2) = chars.next() {
                // 0 -- 7 に変換
                let x = u32::from(c1) - 0x41;
                let y = u32::from(c2) - 0x31;
                if 0 <= x && x <= 7 && 0 <= y && y <= 7 {
                    return Some(Move::Put {
                        x,
                        y,
                    });
                }
            }
        }
        None
    }
}
