// options: parse options.
use getopts;

use std::error::Error;

#[derive(Debug)]
pub struct Opts {
    // protocols
    pub port: u16,
    pub host: String,
    pub name: String,
    // strategies
    pub ending_turns: u32,
}

pub fn parse(args: Vec<String>) -> Result<Opts, String>{

    // define options
    let mut opts = getopts::Options::new();
    opts.optopt("p", "port", "Port of othello server.", "PORT");
    opts.optopt("h", "host", "Host of othello server.", "HOST");
    opts.optopt("n", "name", "Name of client.", "NAME");
    opts.optopt("e", "ending", "Number of turns to run in ending strategy (default: 10)", "NUMBER");

    // parse options
    let args = try!(opts.parse(args).or_else(geterr));

    // get options
    let port = try!(args.opt_str("port").map_or(Ok(3000), |s| s.parse()).or_else(geterr));
    let host = args.opt_str("host").unwrap_or(String::from("localhost"));
    let name = args.opt_str("name").unwrap_or(String::from("client"));
    let ending_turns = try!(args.opt_str("ending").map_or(Ok(12), |s| s.parse()).or_else(geterr));

    Ok(Opts {
        port,
        host,
        name,
        ending_turns,
    })
}

fn geterr<T: Error, S>(err: T) -> Result<S, String>{
    Err(err.description().to_owned())
}

