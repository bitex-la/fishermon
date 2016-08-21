#![feature(custom_derive, plugin)]
/* The fishermon strategy is setting up 2 nets both for buying and selling.
 * We start by delimiting a series of prices to serve as 'steps'.
 * Then we set up a strategy to take a stronger position at each step.
 * The progression of the orders to be placed can be either linear or exponential.
 */

extern crate bitex;
#[macro_use]
extern crate fishermon;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate toml;
extern crate rustc_serialize;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use fishermon::{Trader, Strategy};

#[derive(RustcDecodable, Debug)]
struct Config {
    api_key: String,
    sleep_for: u64,
    production: bool,
    bids: Strategy,
    asks: Strategy,
}

fn load_config() -> Config {
    let path = Path::new("fishermon.toml");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    let config = match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => toml::decode(s.parse().unwrap()),
    };

    config.expect("Config format was invalid")
}

fn main() {
    env_logger::init().unwrap();
    info!("starting up");

    let config = load_config();

    let api = if config.production {
      bitex::Api::prod().key(&config.api_key)
    } else {
      bitex::Api::sandbox().key(&config.api_key)
    };

    let trader = Trader::new(api, config.sleep_for, 300, config.bids, config.asks);

    loop {
      trader.trade()
    }
}
