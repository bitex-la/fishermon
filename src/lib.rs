extern crate rustc_serialize;
#[macro_use]
extern crate decimal;
#[macro_use]
extern crate log;
extern crate bitex;
use std::time::Duration;
use std::thread;

pub mod strategy;
pub use strategy::Strategy;
use bitex::{Api, Bid, Ask, StatusCode, Order};
use bitex::curs::{CursResult, CursError};
use bitex::curs::serde::d128;

#[derive(Debug)]
pub struct Trader<'a> {
    pub api: Api<'a>,
    pub sleep_for: u64,
    pub cooldown: u64,
    pub bids_config: Strategy,
    pub asks_config: Strategy,
}

impl<'a> Trader<'a> {
    pub fn new(api: Api<'a>, sleep: u64, cool: u64, bids: Strategy, asks: Strategy) -> Trader<'a> {
        assert!(bids.price_delta.is_negative(), "Bids need negative delta");
        assert!(asks.price_delta.is_positive(), "Asks need positive delta");
        Trader{
            api: api,
            sleep_for: sleep,
            cooldown: cool,
            bids_config: bids,
            asks_config: asks,
        }
    }

    fn with_retry<F: Fn() -> CursResult<A>, A>(&self, func: F) -> CursResult<A> {
        thread::sleep(Duration::from_millis(self.cooldown));
        func().or_else(|err|{
            info!("Last operation errored with \n {:?}", err);
            match err {
                CursError::Status(ref r) if r.status != StatusCode::UnprocessableEntity =>
                    self.with_retry(func),
                CursError::Network(_) => self.with_retry(func),
                e => Err(e)
            }
        })
    }

    fn pairs_to_orders<F, A>(&self, pairs: Vec<(d128, d128)>, func: F) -> Vec<CursResult<A>>
        where F: Fn(d128, d128) -> CursResult<A>
    {
        pairs.into_iter().map(|(a,p)|{
            self.with_retry(|| func(a,p))
        }).collect()
    }

    pub fn place_bids(&self, pairs: Vec<(d128, d128)>) -> Vec<CursResult<Bid>> {
        self.pairs_to_orders(pairs, |a,p|{
            info!("Placing Bid ${} @ ${}", a, p);
            self.api.bids().create(a,p)
        })
    }

    pub fn place_asks(&self, pairs: Vec<(d128, d128)>) -> Vec<CursResult<Ask>> {
        self.pairs_to_orders(pairs, |a,p|{
            info!("Placing Ask {} BTC @ ${}", a, p);
            self.api.asks().create(a,p)
        })
    }

    pub fn clear_all_orders(&self){
        info!("Clearing orders");
        loop {
            let orders = self.with_retry(|| self.api.orders()).unwrap();
            if orders.is_empty() { break }
            for o in orders.into_iter() { 
              match o {
                Order::Bid(o) => {self.api.bids().cancel(o.id).unwrap();},
                Order::Ask(o) => {self.api.asks().cancel(o.id).unwrap();}
              };
            };
            thread::sleep(Duration::from_millis(self.cooldown));
        }
    }

    pub fn trade(&self){
        self.clear_all_orders();
        info!("Starting trade");
        let book = self.with_retry(|| self.api.orderbook()).unwrap();
        let profile = self.with_retry(|| self.api.profile()).unwrap();
        self.place_bids(self.bids_config.build_orders(book.bids[0].0));
        self.place_asks(self.asks_config.build_orders(book.asks[0].0));
        thread::sleep(Duration::from_millis(self.sleep_for));
    }
}
