extern crate itertools;
extern crate bitex;

#[derive(RustcDecodable, Debug)]
pub struct Strategy {
    pub total_amount: f64,
    pub min_size: f64,
    pub price_delta: f64,
    pub price_growth: f64,
    pub amount_growth: f64,
    pub count: i64,
}

impl Strategy {
    fn order_amounts(&self) -> Vec<f64> {
        let mut sum = 0.0;
        let mut sequence = vec![];
        
        for i in 1..(self.count + 1) {
          let val = (i as f64).powf(self.amount_growth);
          sum += val;
          sequence.push(val);
        }

        sequence.iter().map(|&p| p * self.total_amount / sum).collect()
    }

    fn order_prices(&self, start: f64) -> Vec<f64>{
        let mut sequence = vec![];
        
        for i in 1..(self.count + 1) {
          sequence.push((i as f64).powf(self.price_growth));
        }

        sequence.iter().map(|&p| start + (p * self.price_delta / sequence.last().unwrap())).collect()
    }

    pub fn build_orders(&self, start: f64) -> Vec<(f64, f64)> {
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
            total_amount: 500.0,
            min_size: 1.0,
            price_delta: 50.0,
            price_growth: 2.0,
            amount_growth: 1.5,
            count: 4,
        };
        assert_eq!(
            strategy.build_orders(100.0),
            vec![
                (29.36930093376719, 103.125),
                (83.06892739590073, 112.5),
                (152.60736420019452, 128.125),
                (234.9544074701375, 150.0),
            ]
        );
    }

    #[test]
    fn test_build_with_cutoff(){
        // With such a low total_amount the first amount in the sequence will
        // be cut off because is less than the min_size (0.46~ < 1)
        let strategy = Strategy {
            total_amount: 8.0,
            min_size: 1.0,
            price_delta: 50.0,
            price_growth: 2.0,
            amount_growth: 1.5,
            count: 4,
        };
        assert_eq!(
            strategy.build_orders(100.0),
            vec![
                (1.3291028383344117, 112.5),
                (2.4417178272031124, 128.125),
                (3.7592705195222003, 150.0),
            ]
        );
    }

    #[test]
    fn test_build_bids(){
        let strategy = Strategy {
            total_amount: 500.0,
            min_size: 1.0,
            price_delta: -50.0,
            price_growth: 2.0,
            amount_growth: 1.5,
            count: 4,
        };
        assert_eq!(
            strategy.build_orders(150.0),
            vec![
                (29.36930093376719, 146.875),
                (83.06892739590073, 137.5),
                (152.60736420019452, 121.875),
                (234.9544074701375, 100.0)
            ]
        );
    }
}
