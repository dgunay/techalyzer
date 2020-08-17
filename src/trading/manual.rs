use super::tradingmodel::{Trades, TradingModel};
use crate::Date;
use crate::{backtester::Position, signals::SignalsIter};
use crate::{
    marketdata::prices::Prices,
    signals::{
        bollingerbandssignals::BBSignalsIter, macdsignals::MACDSignalsIter,
        relativestrengthindexsignals::RSISignalsIter, Signal,
    },
};
use derive_more::Display;
use dg_ta::indicators::SimpleMovingAverage;
use std::collections::BTreeMap;

pub enum Error {
    NoSignalAvailable,
}

pub struct ManualTradingModel {
    // TODO: make this a max and let the bot decide dynamically how many shares
    // to trade
    shares: u64,

    /// How far the signal needs to be from 0 in order to make a trade. For
    /// example, if the dead zone is 0.2, only an average signal of less than
    /// -0.2 or greater than 0.2 will cause the model to go short or long.
    dead_zone: Signal,

    /// The tendency for the algorithm to be bullish or bearish by adding or
    /// subtracting from the signal before determining a trade.
    disposition: Signal,
}

impl ManualTradingModel {
    pub fn new(shares: u64, dead_zone: f64, disposition: f64) -> Self {
        Self {
            shares,
            dead_zone: dead_zone.into(),
            disposition: disposition.into(),
        }
    }

    pub fn set_shares(&mut self, shares: u64) {
        self.shares = shares;
    }
}

impl Default for ManualTradingModel {
    fn default() -> Self {
        Self::new(1000, 0.0, 0.0)
    }
}

#[derive(Debug, Display)]
pub enum ManualTradingModelError {
    NotConvertibleFromStr(String),
}

#[allow(dead_code)]
enum MarketState {
    Trending,
    Oscillating,
}

pub fn average_slope(_prices: &Prices, _sma: SimpleMovingAverage) -> f64 {
    todo!()
}

impl ManualTradingModel {
    #[allow(dead_code)]
    fn current_market_state(&self, prices: &Prices, _today: &Date) -> MarketState {
        // Take the average slope of some N-day moving average, perhaps 75
        // TODO: parameterize trend checker window instead of hardcoding 75
        let sma = SimpleMovingAverage::new(75).expect("Couldn't construct SMA");
        match average_slope(&prices, sma) {
            slope if slope >= 0.4 || slope <= -0.4 => MarketState::Trending,
            _ => MarketState::Oscillating,
        }
    }
}

impl TradingModel for ManualTradingModel {
    type Error = ManualTradingModelError;

    fn get_trades(self, prices: &Prices) -> Result<Trades, Self::Error> {
        // Make a bin of technical indicators to use - 2 trending, 2 oscillating.

        let mut rsi = RSISignalsIter::default();
        let mut bb = BBSignalsIter::default();
        let mut macd = MACDSignalsIter::default();

        let mut trades = BTreeMap::new();
        for (day, price) in prices.iter() {
            // TODO: make the market trend cause the algo to favor trend or
            // oscillating indicators
            // let market_state = match self.current_market_state(&prices, &day) {
            //     MarketState::Trending => todo!("Favor trend indicators"),
            //     MarketState::Oscillating => todo!("Favor oscillating indicators"),
            // };
            let signals = vec![rsi.next(*price).0, bb.next(*price).0, macd.next(*price).0];
            let sum: f64 = signals.iter().map(|s| s.val).sum();
            let signal_average = (sum / signals.len() as f64) + self.disposition;

            // Consult the indicators' consensus.
            let trade = match signal_average {
                avg if avg > self.dead_zone.into() => Position::Long(self.shares),
                avg if avg < -(f64::from(self.dead_zone)) => Position::Short(self.shares),
                _ => Position::Out, // TODO: should I hold instead?
            };

            // Make a trade.
            trades.insert(*day, trade);
        }

        Ok(Trades { trades })
    }
}

#[cfg(test)]
mod tests {
    use super::ManualTradingModel;
    use crate::Date;
    use crate::{
        backtester::Position, marketdata::prices::Prices, trading::tradingmodel::TradingModel,
        util::TimeSeries,
    };
    use chrono::Duration;
    use std::collections::BTreeMap;

    /// Creates a month of flat Prices
    fn fixture_setup() -> Prices {
        let start = Date::from_ymd(2012, 1, 2);
        let end = Date::from_ymd(2012, 2, 2);
        let mut dt = start;
        let mut entries = BTreeMap::new();
        while dt <= end {
            entries.insert(dt, 30.0);
            // dt = dt.and_hms(1, 1, 1) + Duration::days(1);
            dt = dt + Duration::days(1);
        }

        Prices {
            map: entries,
            symbol: "jpm".to_string(),
        }
    }

    // TODO: a good macro would really help make things easier
    // macro_rules! date_series_from_vec {
    //     ($y:literal, $m:literal, $d:literal, $items:tt) => {
    //         let mut start = Date::from_ymd($y, $m, $d);
    //         // let mut dt = start;
    //         let mut entries = BTreeMap::new();
    //         // for i in vec![$items] {
    //         $items(
    //             entries.insert(dt, $i);
    //             dt = dt + Duration::days(1);
    //         )*
    //         // }

    //         entries
    //     };
    // }

    #[test]
    fn test_manual_trader() {
        let map: TimeSeries<f64> = vec![
            (Date::from_ymd(2020, 1, 1), 1.0),
            (Date::from_ymd(2020, 1, 2), 2.0),
            (Date::from_ymd(2020, 1, 3), 3.0),
            (Date::from_ymd(2020, 1, 4), 4.0),
            (Date::from_ymd(2020, 1, 5), 5.0),
            (Date::from_ymd(2020, 1, 6), 6.0),
            (Date::from_ymd(2020, 1, 7), 7.0),
            (Date::from_ymd(2020, 1, 8), 8.0),
        ]
        .iter()
        .cloned()
        .collect();

        let prices = Prices {
            map: map,
            symbol: "jpm".to_string(),
        };

        let algo = ManualTradingModel::default();
        let trades: Vec<Position> = algo
            .get_trades(&prices)
            .unwrap()
            .trades
            .values()
            .into_iter()
            .cloned()
            .collect();

        // First position should pretty much always be Out.
        assert!(trades[0] == Position::Out);
        assert!(trades[1..].iter().all(|p| *p != Position::Out));
    }

    #[test]
    fn test_manual_trader_disposition() {
        // prices completely flat for one month
        let prices = fixture_setup();

        // set to perma-bear mode
        let algo = ManualTradingModel::new(1, 0.0, -1.0);
        let trades = algo.get_trades(&prices).unwrap();
        assert!(trades.trades.iter().all(|t| *t.1 == Position::Short(1)));

        // perma-bull mode
        let algo = ManualTradingModel::new(1, 0.0, 1.0);
        let trades = algo.get_trades(&prices).unwrap();
        assert!(trades.trades.iter().all(|t| *t.1 == Position::Long(1)));
    }
}
