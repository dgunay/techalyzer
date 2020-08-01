Neither the outputs of Techalyzer nor the views of its contributors constitutes
professional or financial advice.

# Techalyzer

After taking Machine Learning for Trading at Georgia Tech, I wanted to try
computational investing/algorithmic trading on my own time, with unrestricted 
tooling. This repo contains that work.

A few goals:

* Have something I can easily generate trading insights from for fun (and if I'm 
  lucky, profit)
* Be able to use Rust for performance

## Design

I want to be able to automatically fetch daily market data (maybe even minute to
minute data if possible) from one or more APIs, and then for a given stock:

* Backtest and optimize a trading strategy using several indicators and either a 
  handwritten algorithm or a machine learning model.
* Using a trained model or algorithm, recommend a course of action given new 
  data each day.

This will involve implementing or sourcing:

* A market simulator (how portfolio value changes depending on when you trade a
  given security)
* Implementing a buy-and-hold benchmark
* Implementations of several technical indicators (at least RSI, BB, MAC, etc)
* One or more ML algorithms well-suited to maximizing portfolio return given
  technical signals, or an optimizer that can tune the signals for a handwritten
  trading strategy

Other annoying subproblems that may crop up:
* Dealing with time series data effectively (including gaps introduced by holidays,
  weekends, etc). The [`bdays`][bdays] crate may help with that.
* Having a good plotting solution to sanity check the trading bot
  * Currently Techalyzer serializes to JSON via `serde_json` and then uses 
    matplotlib (see [`plot_signals.py`](scripts/plotting/plot_signals.py))
* 

[bdays]: https://docs.rs/bdays/0.1.1/bdays/index.html

The basic idea:

```
+-------------+    +------------------+    +--------------------------------+
| Market Data |--->| Trading Strategy |--->| Insights (Buy/Sell/Do Nothing) |
+-------------+    +------------------+    +--------------------------------+
  Finance API        ML/bespoke algo           Informs human trader 

```

There is no initial intent to make this a highly scalable or distributed 
architecture, it is simply a CLI app for now.

## Known Issues

Compiling in Windows is a bit of a pain in the ass because the `peroxide` crate's
`dataframe` feature uses `netcdf`, 
[which has a linking bug at the moment](https://github.com/Axect/Peroxide/issues/16). 
Can be solved by either manually adding it to the linking path, or by building 
it in WSL. To get it for Windows, [install it from here][netcdf] and then add
it to your PATH.

[netcdf]: https://www.unidata.ucar.edu/software/netcdf/docs/winbin.html

## Things of Interest

* [Polars DataFrame](https://github.com/ritchie46/polars) looks like a great
  fit as a competitor to pandas, should the need arise.

## Random Backlog

* Integration tests to write in techalyzer.rs:
  * TODO: `--file file` doesn't exist
  * TODO: `--file file` isn't valid JSON/CSV/etc.

### For funsies:

* Integrate a sentiment analyzer for r/wallstreetbets or various investing forums
