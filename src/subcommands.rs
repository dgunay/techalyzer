//! Subcommands for the Techalyzer program. They are split off to allow for
//! easier integration and end-to-end testing.

use crate::Date;
use crate::{
    backtester::BackTester,
    error::TechalyzerError,
    marketdata::prices::Prices,
    output::{
        SupportedIndicators, TechalyzerBacktestOutput, TechalyzerEntry, TechalyzerPrintOutput,
    },
    signals::{
        bollingerbandssignals::BBSignalsIter, macdsignals::MACDSignalsIter,
        relativestrengthindexsignals::RSISignalsIter, Output, Signal, SignalsIter,
    },
    trading::{
        buyandhold::BuyAndHold,
        dtmodel::{DecisionTreeError, DecisionTreeTrader, Trained},
        manual::ManualTradingModel,
        tradingmodel::TradingModel,
        SupportedTradingModel,
    },
};
use std::{fs::File, path::PathBuf};

/// Using price time series info and a technical indicator, prints the buy/sell
/// signals, the indicator outputs, and prices to STDOUT as JSON.
pub fn print(prices: Prices, indicator: SupportedIndicators) -> Result<(), TechalyzerError> {
    // TODO: evaluate/benchmark signal generation using ndarray vs Vec<f64>

    // Calculate the technical indicator outputs and signals
    // TODO: allow parameters for each indicator
    // FIXME: is there any way we can avoid heap allocating/dynamic dispatch?
    let mut sig_iter: Box<dyn SignalsIter> = match indicator {
        SupportedIndicators::BollingerBands => Box::new(BBSignalsIter::default()),
        SupportedIndicators::RelativeStrengthIndex => Box::new(RSISignalsIter::default()),
        SupportedIndicators::MACD => Box::new(MACDSignalsIter::default()),
    };

    let results: Vec<(Signal, Output)> = prices.iter().map(|p| sig_iter.next(*p.1)).collect();

    let mut m = std::collections::BTreeMap::new();
    for (i, (date, price)) in prices.iter().enumerate() {
        m.insert(
            *date,
            TechalyzerEntry {
                price: *price,
                signal: results[i].0,
                output: results[i].1.clone(),
            },
        );
    }

    let output = TechalyzerPrintOutput {
        symbol: prices.symbol,
        indicator,
        map: m,
    };

    // TODO: genericize the output stream to allow for writing to a file
    print!("{}", serde_json::to_string(&output)?);
    Ok(())
}

/// Trains a machine learning classifier using a Prices time series, across the given
/// set of train_dates, using a list of technical indicators. The model will be
/// serialized to a binary file for later use.
///
/// ### Arguments
///
/// * `prices` - Prices dataset.
/// * `train_dates` - Date range to train the model on. Should be `horizon` days
/// less than the end of `prices`.
/// * `signal_generators` - Technical Indicators to serve as features for the model.
/// * `horizon` - During labelling, returns from this many days in the future are
/// used. If the returns are positive or negative,
/// * `outpath` - Where to save the serialized model file for later use.
pub fn train(
    prices: Prices,
    train_dates: Vec<Date>,
    signal_generators: Vec<SupportedIndicators>,
    horizon: u32,
    // TODO: add threshold as a param here
    out_path: PathBuf,
) -> Result<(), TechalyzerError> {
    let gens = signal_generators.iter().map(|f| f.into()).collect();
    let model = train_model(&prices, train_dates, gens, horizon)?;

    let file = File::create(out_path)?;
    bincode::serialize_into(file, &model)?;
    Ok(())
}

impl From<SupportedIndicators> for Box<dyn SignalsIter> {
    fn from(s: SupportedIndicators) -> Self {
        match s {
            SupportedIndicators::BollingerBands => Box::new(BBSignalsIter::default()),
            SupportedIndicators::RelativeStrengthIndex => Box::new(RSISignalsIter::default()),
            SupportedIndicators::MACD => Box::new(MACDSignalsIter::default()),
        }
    }
}

fn train_model(
    prices: &Prices,
    train_dates: Vec<Date>,
    signal_generators: Vec<Box<dyn SignalsIter>>,
    horizon: u32,
) -> Result<DecisionTreeTrader<Trained>, DecisionTreeError> {
    // TODO: either load a model or train a new one right here.
    let model = DecisionTreeTrader::new(signal_generators)?;
    // TODO: don't hardcode threshold.
    let trained = model.train(prices, train_dates, horizon, 0.03)?;

    Ok(trained)
}

/// Backtests the performance of a trading model over a given set of price data.
/// The backtest results are written to STDOUT as JSON.
///
/// ### Arguments
/// * `prices` - Dataset to test the trading model over.
/// * `trading_model` - One of the trading models supported by Techalyzer.
/// * `model_file` - If given, the file from which to load the trading model.
/// * `cash` - How much cash the trading model starts with.
pub fn backtest(
    prices: Prices,
    trading_model: SupportedTradingModel,
    model_file: Option<PathBuf>,
    cash: f64,
) -> Result<(), TechalyzerError> {
    // TODO: allow parameters for the models here.

    let trades = match &trading_model {
        // TODO: don't unwrap
        SupportedTradingModel::BuyAndHold => BuyAndHold::default().get_trades(&prices).unwrap(),
        SupportedTradingModel::ManualTradingAlgo => {
            ManualTradingModel::default().get_trades(&prices)?
        }
        SupportedTradingModel::MachineLearningModel => {
            let model: DecisionTreeTrader<Trained> = match model_file {
                Some(path) => bincode::deserialize(std::fs::read(path)?.as_slice())?,
                None => return Err(TechalyzerError::NoModelFileSpecified),
            };
            model.get_trades(&prices)?
        }
    };

    // TODO: have a way for the model to tell us its signal data

    // Give the backtester the trades
    let performance = BackTester::new(trades.clone(), &prices, cash)?.backtest()?;

    let bench_trades = BuyAndHold::default().get_trades(&prices)?;
    let bench_perf = BackTester::new(bench_trades, &prices, cash)?.backtest()?;

    let total_return = performance.total_return()?;

    let symbol = prices.symbol.clone();
    let output = TechalyzerBacktestOutput {
        performance,
        total_return,
        trades,
        model_name: trading_model.to_string(),
        symbol,
        prices,
        benchmark: bench_perf,
    };

    // Serialize the backtest
    match serde_json::to_writer(std::io::stdout(), &output) {
        Ok(_) => (),
        Err(e) => return Err(TechalyzerError::Generic(e.to_string())),
    };

    // TODO: other stats like daily return here
    // print!(
    //     "Total return: {}",
    //     output.total_return
    // );

    Ok(())
}
