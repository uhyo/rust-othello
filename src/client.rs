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
    static ref REND: Regex = Regex::new(r"^END ((?i)WIN|LOSE|TIE) (\d+) (\d+)(.*)").unwrap();
    // MOVEコマンド
    static ref RMOVE: Regex = Regex::new(r"^MOVE (PASS|[A-H][1-8])").unwrap();
    // ACKコマンド
    static ref RACK: Regex = Regex::new(r"^ACK (-?\d+)").unwrap();
}

#[derive(Debug)]
pub struct Client<B, S>
    where B: Board + Clone, S: Strategy {
    name: String,
    stream: BufReader<TcpStream>,
    // my color
    color: Turn,
    // remaining time
    time: i32,
    // board
    board: B,
    // strategy
    strategy: S,
}

impl<B, S> Client<B, S> where B: Board + Clone, S: Strategy {
    pub fn new(opts: &options::Opts, board: B, strategy: S) -> io::Result<Self>{
        let name = opts.name.clone();
        let stream = TcpStream::connect((opts.host.as_ref(), opts.port))?;
        info!("Connected to the server");

        let stream = BufReader::new(stream);
        Ok(Client {
            name,
            stream,
            color: Turn::Black,
            time: 0,
            board,
            strategy,
        })
    }

    // perform init 
    pub fn run(&mut self) -> io::Result<()> {
        // first, we send the `NAME` command
        writeln!(self.stream.get_mut(), "OPEN {}", self.name)?;
        debug!("Registered as client {}", self.name);

        self.wait_for_game()
    }

    // 'waiting for game' state
    fn wait_for_game(&mut self) -> io::Result<()> {
        trace!("State: wait_for_game");
        let mut buf = String::new();
        self.stream.read_line(&mut buf)?;
        if RBYE.is_match(&buf) {
            // BYEコマンドを受け取った
            trace!("{}", buf.trim());
            info!("Connection closed by the server");
            return Ok(());
        }
        if let Some(caps) = RSTART.captures(&buf) {
            trace!("{}", buf.trim());
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
            self.board.reset();
            self.strategy.reset();

            if self.color == Turn::Black {
                return self.my_turn(None);
            } else {
                return self.opponent_turn();
            }
        }

        debug!("{}", buf.trim());
        warn!("Invalid command sent from the server");

        Ok(())
    }
    // opponent's turn
    fn opponent_turn(&mut self) -> io::Result<()>{
        trace!("State: opponent_turn");
        // 相手ターンなので通信を待機
        // TODO 相手ターンでも探索したらいいのでは?
        let mut buf = String::new();
        self.stream.read_line(&mut buf)?;

        if let Some(caps) = REND.captures(&buf) {
            trace!("{}", buf.trim());
            // 結果表示
            let mut buf = String::new();
            caps.expand("Game ended: $1 ($2/$3) - $4", &mut buf);
            info!("{}", buf.trim());

            return self.wait_for_game();
        }
        if let Some(caps) = RMOVE.captures(&buf) {
            trace!("{}", buf.trim());
            let mv = caps.get(1).unwrap().as_str();
            if let Some(mv) = parse_move(&mv) {
                // 動かす
                return match self.board.apply_move(mv) {
                    Ok(()) => {
                        trace!("\n{}", self.board.pretty_print());
                        // 次のターンへ
                        self.my_turn(Some(mv))
                    },
                    Err(s) => {
                        // おかしい着手が来たぞ
                        warn!("Invalid move sent from the server - {}", s);
                        Ok(())
                    },
                }
            }
            trace!("Failed to parse move"); 
        }
        // 変なレスポンス
        debug!("{}", buf.trim());
        warn!("Invalid command sent from the server");
        Ok(())
    }
    // my turn
    fn my_turn(&mut self, last_move: Option<Move>) -> io::Result<()>{
        trace!("State: my_turn");
        // 自分の番なので手番をアレする
        let mv = self.strategy.play(&self.board, last_move, self.time);

        let s = serialize_move(mv);
        // 送信
        writeln!(self.stream.get_mut(), "MOVE {}", s)?;
        trace!("MOVE {} ({})", s, mv);

        // 手元の盤面も更新
        if let Err(s) = self.board.apply_move(mv) {
            warn!("Invalid move produced from our strategy: {}", s);
        }

        trace!("\n{}", self.board.pretty_print());

        // ACKを待つ
        let mut buf = String::with_capacity(20);
        self.stream.read_line(&mut buf)?;

        if let Some(caps) = RACK.captures(&buf) {
            // ACK来た
            trace!("{}", buf.trim());
            let time: i32 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap();
            self.time = time;
            // これは相手の番だ
            return self.opponent_turn();
        }
        if let Some(caps) = REND.captures(&buf) {
            trace!("{}", buf.trim());
            // ゲーム終わった
            let mut buf = String::new();
            caps.expand("Game ended: $1 ($2/$3) - $4", &mut buf);
            info!("{}", buf.trim());

            return self.wait_for_game();
        }
        // 変なの来た
        debug!("{}", buf.trim());
        warn!("Invalid command sent from the server");
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
                let x = (c1 as u8) - 0x41;
                let y = (c2 as u8) - 0x31;
                if x <= 7 &&  y <= 7 {
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

fn serialize_move(mv: Move) -> String{
    match mv {
        Move::Pass => String::from("PASS"),
        Move::Put {x, y} => {
            let mut res = String::with_capacity(3);
            res.push(char::from(0x41 + x));
            res.push(char::from(0x31 + y));
            return res;
        }
    }
}
