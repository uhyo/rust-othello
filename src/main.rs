extern crate othello;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::env;
use std::process;
use std::error::Error;

use othello::options;
use othello::client::Client;
use othello::board;
use othello::strategy;

fn main() {
    pretty_env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();
    let opts = options::parse(args).unwrap_or_else(show_and_exit);

    let board = board::make_board();
    let strategy = strategy::make_strategy();

    let mut client = Client::new(&opts, board, strategy).unwrap_or_else(show_err_and_exit);

    client.run().unwrap();

}

fn show_and_exit<R>(err: String) -> R{
    error!("{}", err);
    process::exit(1)
}
fn show_err_and_exit<R, T: Error>(err: T) -> R{
    error!("{}", err.description());
    process::exit(1)
}
