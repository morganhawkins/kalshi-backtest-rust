use super::data_types::TestResult;
use crate::models::gbm;
use crate::read_data::data_types::{CoinbaseRecord, KalshiRecord};
use crate::stream_data::market_stream::PairMarketStream;
use std::cell::RefCell;

struct UnderlyingOrder {
    exec_price: f64,
    quantity: f64,
}

#[allow(dead_code)]
struct GreekExp {
    delta: Option<f64>,
    gamma: Option<f64>,
    vega: Option<f64>,
    theta: Option<f64>,
}

pub struct DeltaHedge<'a> {
    // data feeder
    stream: PairMarketStream<'a>,
    under_terminal: f64,
    strike: f64,
    init_cash: f64,
    // track positions
    under_orders: RefCell<Vec<UnderlyingOrder>>,
    under_position: RefCell<f64>,
    deriv_position: RefCell<f64>,
    cash: RefCell<f64>,
    // hyper parameters
    max_under_pos: f64,
    min_tte_hedge: f64, 
}

impl<'a> DeltaHedge<'a> {
    pub fn new(
        stream: PairMarketStream<'a>,
        max_under_pos: f64,
        min_tte_hedge: f64,
        init_cash: f64,
    ) -> Self {
        let under_terminal = stream.under_data.last().unwrap().price;
        let strike = stream.deriv_data.strike;

        Self {
            stream: stream,
            under_terminal: under_terminal,
            strike: strike,
            init_cash: init_cash,
            under_orders: RefCell::new(Vec::new()),
            under_position: RefCell::new(0.0),
            deriv_position: RefCell::new(0.0),
            cash: RefCell::new(init_cash),
            max_under_pos: max_under_pos,
            min_tte_hedge: min_tte_hedge,
        }
    }

    fn cycle(&self) {
        self.stream.cycle();
    }

    fn place_under_order(&self, quantity: f64, cb_record: &CoinbaseRecord) {
        self.under_orders.borrow_mut().push(UnderlyingOrder {
            exec_price: cb_record.price,
            quantity: quantity,
        });
        *self.under_position.borrow_mut() += quantity;
    }

    fn zero_hedge(&self, cb_record: &CoinbaseRecord) {
        let under_pos = *self.under_position.borrow();
        if under_pos != 0.0 {
            self.under_orders.borrow_mut().push(UnderlyingOrder {
                exec_price: cb_record.price,
                quantity:  under_pos,
            });
            *self.under_position.borrow_mut() = 0.0;
        }
    }

    fn place_deriv_order(&self, quantity: f64, kl_record: &KalshiRecord) {
        let exec_price = ((kl_record.ask + kl_record.bid) as f64) / 200.0;
        *self.deriv_position.borrow_mut() += quantity;
        *self.cash.borrow_mut() -= quantity * exec_price;
    }

    fn reconcile_hedge_orders(&self, under_price: f64) {
        let mut accum = 0.0;
        for order in self.under_orders.borrow().iter() {
            accum += order.quantity * (under_price - order.exec_price);
        }

        *self.cash.borrow_mut() += accum;
    }

    fn calc_greek_exp(
        &self,
        cb_record: &CoinbaseRecord,
        kl_record: &KalshiRecord,
    ) -> Option<GreekExp> {
        let deriv_mp = (kl_record.bid + kl_record.bid) as f64 / 200.0;
        let under_mp = cb_record.price;
        let tte = kl_record.tte;
        let iv = gbm::iv(deriv_mp, under_mp, self.strike, 0.0, tte)?;

        Some(GreekExp {
            delta: gbm::delta(under_mp, self.stream.deriv_data.strike, iv, 0.0, tte),
            gamma: gbm::gamma(under_mp, self.stream.deriv_data.strike, iv, 0.0, tte),
            vega: gbm::vega(under_mp, self.stream.deriv_data.strike, iv, 0.0, tte),
            theta: gbm::theta(under_mp, self.stream.deriv_data.strike, iv, 0.0, tte),
        })
    }

    fn adjust_delta_hedge(&self, exposures: GreekExp, cb_record: &CoinbaseRecord) {
        let deriv_position = *self.deriv_position.borrow();
        let under_position = *self.under_position.borrow();
        let min_order = -self.max_under_pos - under_position;
        let max_order = self.max_under_pos - under_position;
        // println!("{under_position:.5} {min_order:.5} {max_order:.5}");
        let delta_err = match exposures.delta {
            Some(delta_) => {
                delta_ * deriv_position + under_position
        },
            None => return,
        };
        if delta_err != 0.0 {
            self.place_under_order((-delta_err).clamp(min_order, max_order), cb_record);
        }
    }

    fn validate_data(&self, kl_record: &KalshiRecord) -> bool {
        if (kl_record.ask - kl_record.bid) > 5 {
            return false;
        } else if (kl_record.ask > 95) || (kl_record.bid < 5) {
            return false
        }
        return true
    }

    fn consume(&self) {
        // attempting to grab data for current time
        let (cb_record, kl_record) = match self.stream.get_records() {
            Some((a, b)) => (a, b),
            None => return,
        };
        // calc exposures, will just return if iv is not able to be computed
        // TODO: last valid iv tracking to lessen timestamps where hedge does not change
        let exposures = match self.calc_greek_exp(cb_record, kl_record) {
            Some(val) => val,
            None => return,
        };

        // buy a derivative if port is empty and spread is small
        if *self.deriv_position.borrow() == 0.0 && ((kl_record.ask - kl_record.bid) < 10) {
            self.place_deriv_order(1.0, kl_record);
        };

        if self.validate_data(kl_record) {
            self.adjust_delta_hedge(exposures, cb_record);
        } else if (kl_record.tte <= self.min_tte_hedge){
            self.zero_hedge(cb_record);
        }
    }

    pub fn test(&self) -> TestResult {
        loop {
            // consume data and cycle timer
            self.consume();
            self.cycle();

            //break if timer is done
            if let None = self.stream.get_time() {
                break;
            } else {
            }
        }

        // recocnciles hedge orders and updates cash
        self.reconcile_hedge_orders(self.under_terminal);
        // getting terminla cash and derivative position value
        let end_cash = *self.cash.borrow();
        let end_deriv_val =
            f64::from(self.under_terminal >= self.strike) * (*self.deriv_position.borrow());
        let terminal_value = end_cash + end_deriv_val;

        return TestResult {
            init_value: self.init_cash,
            terminal_value: terminal_value,
            deriv_term_value: end_deriv_val,
            strike: self.strike,
            start_ts: self.stream.deriv_data.start_ts,
            expir_ts: self.stream.deriv_data.expir_ts,
        };
    }
}
