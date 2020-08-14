//! Subcommands for the Techalyzer program.

use crate::Date;
use crate::{
    backtester::BackTester,
    error::TechalyzerError,
    marketdata::prices::Prices,
    output::{
        SupportedIndicators, TechalyzerBacktestOutput, TechalyzerEntry, TechalyzerPrintOutput,
    },
    signals::{
        bollingerbandssignals::{BBSignalsIter, BollingerBandsSignals},
        macdsignals::{MACDSignalsIter, MovingAverageConvergenceDivergenceSignals},
        relativestrengthindexsignals::{RSISignalsIter, RelativeStrengthIndexSignals},
        signals::{Signals, SignalsIter},
    },
    trading::{
        buyandhold::BuyAndHold,
        dtmodel::{DecisionTreeError, DecisionTreeTrader},
        manual::ManualTradingModel,
        tradingmodel::TradingModel,
        SupportedTradingModel,
    },
};
use dg_ta::indicators::{
    BollingerBands, MovingAverageConvergenceDivergence, RelativeStrengthIndex,
};
use std::{fs::File, path::PathBuf};

pub fn print(prices: Prices, indicator: SupportedIndicators) -> Result<(), TechalyzerError> {
    // TODO: evaluate/benchmark signal generation using ndarray vs Vec<f64>

    // Calculate the technical indicator outputs and signals
    // TODO: allow parameters for each indicator
    // FIXME: is there any way we can avoid heap allocating/dynamic dispatch?
    let sigs: Box<dyn Signals> = match indicator {
        SupportedIndicators::BollingerBands => Box::new(BollingerBandsSignals::new(
            &prices,
            BollingerBands::new(20, 2.0)?,
        )),
        SupportedIndicators::RelativeStrengthIndex => Box::new(RelativeStrengthIndexSignals::new(
            &prices,
            RelativeStrengthIndex::new(14).expect("invalid RSI params"),
        )),
        SupportedIndicators::MACD => Box::new(MovingAverageConvergenceDivergenceSignals::new(
            &prices,
            MovingAverageConvergenceDivergence::new(12, 26, 9).expect("Invalid MACD params"),
        )),
    };

    // TODO: sadly output shapes are not all the same, BollingerBandsOutput
    // is a tuple of f64s whereas the other indicators usually just have
    // a single f64 per data point. Can this be reconciled in a pretty way
    // before printing it?

    let mut m = std::collections::BTreeMap::new();
    for (i, (date, price)) in prices.iter().enumerate() {
        m.insert(
            *date,
            TechalyzerEntry {
                price: *price,
                signal: sigs.signals()[i],
                output: sigs.outputs()[i].clone(),
            },
        );
    }

    // TODO: factor out this ugliness or change the datastructures
    // involved to be less gross
    let output = TechalyzerPrintOutput {
        symbol: prices.symbol,
        indicator,
        map: m,
    };

    // TODO: genericize the output stream to allow for writing to a file
    print!("{}", serde_json::to_string(&output)?);
    Ok(())
}

/// Outputs a string to an output buffer, Stdout by default.
// fn print_techalyzer_json(output: &TechalyzerPrintOutput) {}

pub fn train(
    prices: Prices,
    train_dates: Vec<Date>,
    signal_generators: Vec<SupportedIndicators>,
    horizon: u32,
    out_path: PathBuf,
) -> Result<(), TechalyzerError> {
    // TODO: Chop off `horizon` days from the train_dates to reserve for lookahead.

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
) -> Result<DecisionTreeTrader, DecisionTreeError> {
    // TODO: either load a model or train a new one right here.
    let mut model = DecisionTreeTrader::new(signal_generators);

    model.train(prices, train_dates, horizon, 0.03)?;

    Ok(model)
}

pub fn backtest(
    prices: Prices,
    trading_model: SupportedTradingModel,
    cash: f64,
) -> Result<(), TechalyzerError> {
    let trades = match trading_model {
        // TODO: don't unwrap
        SupportedTradingModel::BuyAndHold => BuyAndHold::default().get_trades(&prices).unwrap(),
        SupportedTradingModel::ManualTradingAlgo => {
            ManualTradingModel::default().get_trades(&prices)?
        }
        SupportedTradingModel::MachineLearningModel(_) => {
            let _model: DecisionTreeTrader = todo!();
            // model.get_trades(&prices)?
        }
    };

    // TODO: have a way for the model to tell us its signal data

    // Give the backtester the trades
    let symbol = prices.symbol.clone();
    let mut backtester = match BackTester::new(trades.clone(), &prices, cash) {
        Ok(bt) => bt,
        Err(e) => return Err(TechalyzerError::Generic(e.to_string())),
    };

    // Run the backtest
    let performance = match backtester.backtest() {
        Ok(perf) => perf,
        Err(e) => return Err(TechalyzerError::Generic(e.to_string())),
    };

    let total_return = performance.total_return()?;
    // .expect("Couldn't get total return");

    let output = TechalyzerBacktestOutput {
        performance,
        total_return,
        trades,
        model_name: trading_model.to_string(),
        symbol,
        prices,
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
