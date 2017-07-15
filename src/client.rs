// ISOP (IS Othello Protocol) client.

use std::net::TcpStream;
use std::io;
use options;

#[derive(Debug)]
pub struct Client{
    stream: TcpStream,
}

impl Client{
    pub fn connect(opts: &options::Opts) -> io::Result<Self>{
        let target = format!("{}:{}", opts.host, opts.port);
        let stream = TcpStream::connect(target)?;
        Ok(Client {
            stream,
        })
    }
}
