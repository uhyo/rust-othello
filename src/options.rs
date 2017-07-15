// options: parse options.
extern crate getopts;

use std::error::Error;

#[derive(Debug)]
pub struct Opts {
    pub port: i32,
    pub host: String,
}

pub fn parse(args: Vec<String>) -> Result<Opts, String>{

    // define options
    let mut opts = getopts::Options::new();
    opts.reqopt("p", "port", "Port of othello server.", "PORT");
    opts.optopt("h", "host", "Host of othello server.", "HOST");

    // parse options
    let args = try!(opts.parse(args).or_else(geterr));

    // get options
    let port = args.opt_str("port").unwrap();
    let port = try!(port.parse::<i32>().or_else(geterr));
    let host = args.opt_str("host").unwrap_or(String::from("localhost"));

    Ok(Opts {
        port,
        host,
    })
}

fn geterr<T: Error, S>(err: T) -> Result<S, String>{
    Err(err.description().to_owned())
}

