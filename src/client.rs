// ISOP (IS Othello Protocol) client.

use std::net::TcpStream;
use std::io;
use std::io::Write;

use options;

#[derive(Debug)]
pub struct Client{
    name: String,
    stream: TcpStream,
}

impl Client{
    pub fn connect(opts: &options::Opts) -> io::Result<Self>{
        let name = opts.name.clone();
        let stream = TcpStream::connect((opts.host.as_ref(), opts.port))?;
        Ok(Client {
            name,
            stream,
        })
    }

    // perform init 
    pub fn init(&mut self) -> io::Result<()>{
        // first, we send the `NAME` command
        writeln!(self.stream, "OPEN {}", self.name)
    }
}
