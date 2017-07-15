extern crate getopts;

use std::env;
use std::io;
use std::io::Write;
use std::process;
use std::error::Error;

fn main() {
    let opts = options().unwrap_or_else(|err| {
        writeln!(
            io::stderr(),
            "{}", err);
        process::exit(1);
    });

    println!("port: {}", opts.port);
}

struct Opts {
    port: i32,
}

fn options() -> Result<Opts, String>{
    // Get arguments
    let args: Vec<String> = env::args().collect();

    // Define options
    let mut opts = getopts::Options::new();
    opts.reqopt("p", "port", "Port of othello server.", "PORT");

    let args = try!(opts.parse(args).or_else(geterr));

    let port = args.opt_str("port").unwrap();
    let port = try!(port.parse::<i32>().or_else(geterr));

    Ok(Opts {
        port,
    })
}

fn geterr<T: Error, S>(err: T) -> Result<S, String>{
    Err(err.description().to_owned())
}

