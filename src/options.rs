// options: parse options.
extern crate getopts;

use std::error::Error;

#[derive(Debug)]
pub struct Opts {
    pub port: u16,
    pub host: String,
    pub name: String,
}

pub fn parse(args: Vec<String>) -> Result<Opts, String>{

    // define options
    let mut opts = getopts::Options::new();
    opts.reqopt("p", "port", "Port of othello server.", "PORT");
    opts.optopt("h", "host", "Host of othello server.", "HOST");
    opts.optopt("n", "name", "Name of client.", "NAME");

    // parse options
    let args = try!(opts.parse(args).or_else(geterr));

    // get options
    let port = args.opt_str("port").unwrap();
    let port = try!(port.parse().or_else(geterr));
    let host = args.opt_str("host").unwrap_or(String::from("localhost"));
    let name = args.opt_str("name").unwrap_or(String::from("client"));

    Ok(Opts {
        port,
        host,
        name,
    })
}

fn geterr<T: Error, S>(err: T) -> Result<S, String>{
    Err(err.description().to_owned())
}

