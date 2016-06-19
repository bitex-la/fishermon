extern crate itertools;
extern crate bitex;
use self::bitex::curs::serde::d128;
use self::bitex::curs::serde::de::from_primitive::FromPrimitive;

#[derive(RustcDecodable, Debug)]
pub struct Strategy {
    pub total_amount: d128,
    pub min_size: d128,
    pub price_delta: d128,
    pub price_growth: d128,
    pub amount_growth: d128,
    pub count: i64,
}

impl Strategy {
    fn order_amounts(&self) -> Vec<d128> {
        let mut sum = d128!(0);
        let mut sequence = vec![];
        
        for i in 1..(self.count + 1) {
          let val = d128::from_i64(i).unwrap().pow(self.amount_growth);
          sum += val;
          sequence.push(val);
        }

        sequence.iter().map(|&p| p * self.total_amount / sum).collect()
    }

    fn order_prices(&self, start: d128) -> Vec<d128>{
        let mut sequence = vec![];
        
        for i in 1..(self.count + 1) {
          sequence.push(d128::from_i64(i).unwrap().pow(self.price_growth));
        }

        sequence.iter().map(|&p| start + (p * self.price_delta / sequence.last().unwrap())).collect()
    }

    pub fn build_orders(&self, start: d128) -> Vec<(d128, d128)> {
        let prices = self.order_prices(start);
        let amounts = self.order_amounts();
        amounts.into_iter().zip(prices.into_iter()).filter(|&(a,_)| a >= self.min_size).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_asks(){
        let strategy = Strategy {
            total_amount: d128!(500),
            min_size: d128!(1),
            price_delta: d128!(50),
            price_growth: d128!(2),
            amount_growth: d128!(1.5),
            count: 4,
        };
        assert_eq!(
            strategy.build_orders(d128!(100)),
            vec![
                (d128!(29.36930093376719179267728263027333), d128!(103.125)),
                (d128!(83.06892739590073429802931604855244), d128!(112.5)),
                (d128!(152.6073642001945395678751402789877), d128!(128.125)),
                (d128!(234.9544074701375343414182610421866), d128!(150)),
            ]
        );
    }

    #[test]
    fn test_build_with_cutoff(){
        // With such a low total_amount the first amount in the sequence will
        // be cut off because is less than the min_size (0.46~ < 1)
        let strategy = Strategy {
            total_amount: d128!(8),
            min_size: d128!(1),
            price_delta: d128!(50),
            price_growth: d128!(2),
            amount_growth: d128!(1.5),
            count: 4,
        };
        assert_eq!(
            strategy.build_orders(d128!(100)),
            vec![
                (d128!(1.329102838334411748768469056776839), d128!(112.5)),
                (d128!(2.441717827203112633086002244463804), d128!(128.125)),
                (d128!(3.759270519522200549462692176674986), d128!(150)),
            ]
        );
    }

    #[test]
    fn test_build_bids(){
        let strategy = Strategy {
            total_amount: d128!(500),
            min_size: d128!(1),
            price_delta: d128!(-50),
            price_growth: d128!(2),
            amount_growth: d128!(1.5),
            count: 4,
        };
        assert_eq!(
            strategy.build_orders(d128!(150)),
            vec![
                (d128!(29.36930093376719179267728263027333), d128!(146.875)),
                (d128!(83.06892739590073429802931604855244), d128!(137.5)),
                (d128!(152.6073642001945395678751402789877), d128!(121.875)),
                (d128!(234.9544074701375343414182610421866), d128!(100))
            ]
        );
    }
}
