# Technical Analysis Experiments

After taking Machine Learning for Trading at Georgia Tech, I wanted to try
computational investing on my own time, with unrestricted tooling. This repo
contains that work.

A few goals:

* Have something I can easily generate trading insights from for fun (and if I'm 
  lucky, profit)
* Be able to use Rust for performance

## Design

The basic story I guess:

I want to be able to automatically fetch daily market data (maybe even minute to
minute data if possible) from one or more APIs, and then for a given stock:

* Backtest and optimize a trading strategy using several indicators and either a 
  handwritten algorithm or a machine learning model.
* Using a trained model or algorithm, recommend a course of action given new 
  data.

This will involve implementing or sourcing:

* A market simulator (how portfolio value changes depending on when you trade a
  given security)
* Implementations of several technical indicators (at least RSI, BB, MAC, etc)
* One or more ML algorithms well-suited to maximizing portfolio return given
  technical signals, or an optimizer that can tune the signals for a handwritten
  trading strategy

Other annoying subproblems that may crop up:
* Dealing with time series data effectively (including gaps)


The basic idea:

```
+-------------+    +------------------+    +--------------------------------+
| Market Data |--->| Trading Strategy |--->| Insights (Buy/Sell/Do Nothing) |
+-------------+    +------------------+    +--------------------------------+
  Finance API        ML/bespoke algo           Informs human trader 

```

For funsies:

* Integrate a sentiment analyzer for r/wallstreetbets or various investing forums

## Known Issues

Compiling in Windows is a bit of a pain in the ass because the `peroxide` crate's
`dataframe` feature uses `netcdf`, 
[which has a linking bug at the moment](https://github.com/Axect/Peroxide/issues/16). 
Can be solved by either manually adding it to the linking path, or by building 
it in WSL. To get it for Windows, [install it from here][netcdf] and then add
it to your PATH.

[netcdf]: https://www.unidata.ucar.edu/software/netcdf/docs/winbin.html