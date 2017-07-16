// ISOP (IS Othello Protocol) client.

use std::net::TcpStream;
use std::io;
use std::io::{Write, BufRead, BufReader};

use regex::Regex;

use options;
use board::{Board, Turn};
use strategy::Strategy;

#[derive(Debug)]
pub struct Client{
    name: String,
    stream: BufReader<TcpStream>,
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
        lazy_static! {
            // BYEコマンド
            static ref RBYE: Regex = Regex::new(r"^BYE(?:$|\s)").unwrap();
            // STARTコマンド
            static ref RSTART: Regex = Regex::new(r"^START (BLACK|WHITE) (\S+) (\d+)").unwrap();
        }
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
                board.set_turn(Turn::Black);
            } else {
                board.set_turn(Turn::White);
            }
            self.time = time;

            return Ok(());
        }

        debug!("{}", buf.trim());
        warn!("Invalid command sent from the server");

        Ok(())
    }
}
