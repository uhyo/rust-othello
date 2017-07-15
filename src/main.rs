extern crate othello;

use std::env;
use std::io;
use std::io::Write;
use std::process;
use std::fmt::Debug;
use std::error::Error;

use othello::options;
use othello::client::Client;

fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = options::parse(args).unwrap_or_else(show_and_exit);

    let client = Client::connect(&opts).unwrap_or_else(show_err_and_exit);

    println!("{:?}", client);

}

fn show_and_exit<R>(err: String) -> R{
    writeln!(
        io::stderr(),
        "{}", err).unwrap();
    process::exit(1)
}
fn show_err_and_exit<R, T: Error>(err: T) -> R{
    writeln!(
        io::stderr(),
        "{}", err.description()).unwrap();
    process::exit(1)
}
