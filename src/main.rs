extern crate othello;

use std::env;
use std::io;
use std::io::Write;
use std::process;

use othello::options;

fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = options::parse(args).unwrap_or_else(|err| {
        writeln!(
            io::stderr(),
            "{}", err).unwrap();
        process::exit(1);
    });

    println!("{:?}", opts);
}
