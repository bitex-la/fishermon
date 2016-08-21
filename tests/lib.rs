#[macro_use]
extern crate http_stub;
extern crate bitex;
extern crate fishermon;
extern crate env_logger;

use std::sync::{Arc, Mutex};
use http_stub as hs;
use http_stub::hyper::uri::RequestUri as Uri;
use http_stub::regex::Regex;
use bitex::Api;
use fishermon::{Trader, Strategy};

#[test]
fn trade(){
    let cancelled: bool = false;
    let shared = Arc::new(Mutex::new(cancelled));

    /* Server always has 2 bids, 2 asks, always clears everything */
    let url = hs::HttpStub::run(move |stub|{
      if let Uri::AbsolutePath(ref got) = stub.request.uri.clone() {
        let is = |p: &str|{ Regex::new(p).unwrap().is_match(got) };

        if is(&r"cancel"){ *shared.lock().unwrap() = true; }

        let response_body = if is(&r"orders") {
            if *shared.lock().unwrap() {
              "[]"
            } else {
              r#"[
                [1, 1, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01],
                [2, 1, 946685400, 1, 200.00, 20.00, 2000.00, 1, 0, 1.1, "ApiKey#2", 0.01]
              ]"#
            }
        } else if is(&r"/bids/1") {
            r#"[1, 1, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]"#
        } else if is(r"asks/1") {
            r#"[2, 1, 946685400, 1, 200.00, 20.00, 2000.00, 1, 0, 1.1, "ApiKey#2", 0.01]"#
        } else if is(r"profile") {
            r#"{
              "usd_balance": 10000.00,
              "usd_reserved": 2000.00,
              "usd_available": 8000.00,
              "btc_balance": 20.00000000,
              "btc_reserved": 5.00000000,
              "btc_available": 15.00000000,
              "fee": 0.5,
              "btc_deposit_address": "1ABCD",
              "more_mt_deposit_code": "BITEX0000000"
            }"#
        } else if is(r"/api-v1/rest/btc_usd/market/order_book") {
            r#"{"bids":[[500.0,1],[490.0,2]], "asks":[[510.0,1],[520.0,2]]}"#
        } else if is(r"/api-v1/rest/private/bids") {
            r#"[[1, 1, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01 ]]"#
        } else if is(r"/api-v1/rest/private/asks") {
            r#"[ [2, 1, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]]"#
        } else {
            panic!("Unexpected request: {}", got);
        };
        stub.send_body(response_body)
      }
    });
    let api = Api::new(&url);
    let trader = Trader::new(api, 0, 0,
        Strategy {
            total_amount: 500.0,
            min_size: 1.0,
            price_delta: -100.0,
            price_growth: 1.5,
            amount_growth: 1.5,
            count: 3,
        },
        Strategy {
            total_amount: 1.0,
            min_size: 0.005,
            price_delta: 50.0,
            price_growth: 2.0,
            amount_growth: 2.0,
            count: 4,
        }
    );
    trader.trade();
}
