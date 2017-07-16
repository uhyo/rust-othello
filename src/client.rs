// ISOP (IS Othello Protocol) client.

use std::net::TcpStream;
use std::io;
use std::io::{Write, BufRead, BufReader};

use regex::{Regex, RegexSet};

use options;
use board::Board;
use strategy::Strategy;

#[derive(Debug)]
pub struct Client{
    name: String,
    stream: BufReader<TcpStream>,
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
            static ref RSET: RegexSet = RegexSet::new(&[
                r"^BYE\s*$",
            ]).unwrap();
        }
        let mut buf = String::new();
        self.stream.read_line(&mut buf)?;
        let matches = RSET.matches(&buf);
        if matches.matched(0) {
            // BYEコマンドを受け取った
            debug!("Received BYE command");
            info!("Connection closed by the server");
            return Ok(());
        }


        println!("{:?}", matches);

        Ok(())
    }
}
