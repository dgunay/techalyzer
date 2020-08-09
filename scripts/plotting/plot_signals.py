# Output objects are usually like this, with some exceptions:
# {
#   "map": {
#     "2020-03-10": {
#       "signal": 0.0,
#       "price": 100.7
#     },
#   },
#  "symbol": "jpm",
#  "indicator": "RelativeStrengthIndex"
# }

import fileinput
import json
import matplotlib.pyplot as plt
import datetime as dt
import pandas as pd
import jsonschema

# TODO: security symbol in title
# TODO: chart security and indicator in subplots
# TODO: buy/sell signals beyond some threshold as vertical lines
# TODO: schema validation of input https://python-jsonschema.readthedocs.io/en/stable/validate/

# schema = {
#      "type" : "object",
#      "properties" : {
#          "price" : {"type" : "number"},
#          "name" : {"type" : "string"},
#      },
#  }

def signals_subplot(ax: plt.Axes, df: pd.DataFrame):
    ax.set_ylim([-1.0, 1.0])
    colors = df['signal'].map(lambda elem: 'red' if elem < 0 else 'green').values
    ax.bar(df.index, df['signal'], color=colors)
    ax.set_title("Bullish/bearish signals")

def indicator_outputs_subplot(ax: plt.Axes, df: pd.DataFrame, indicator: str):
    ta_df = pd.json_normalize(df['output'])
    ta_df.index = df.index
    for col_name, ta_data in ta_df.iteritems():
        ax.plot(ta_data, label=col_name)
    ax.legend(fontsize='x-small', frameon=False)
    ax.set_title(indicator)

def plot_superimposed_indicator(data: dict, df: pd.DataFrame):
    plt.figure()
    fig, (price_ax, sig_ax) = plt.subplots(2, sharex=True)
    fig.suptitle("{} {} signals".format(data['symbol'].upper(), data['indicator']))

    # Price
    price_ax.plot(df['price'])
    price_ax.set_title("{} price".format(data['symbol'].upper()))

    # Technical Indicator outputs
    indicator_outputs_subplot(price_ax, df, data['indicator'])

    # Buy/sell signals
    signals_subplot(sig_ax, df)

    plt.savefig("{}_{}_signal".format(data['symbol'], data['indicator']))

def plot_separate_indicator(data: dict, df: pd.DataFrame):
    plt.figure()
    fig, (price_ax, ta_ax, sig_ax) = plt.subplots(3, sharex=True)
    fig.suptitle("{} {} signals".format(data['symbol'].upper(), data['indicator']))

    # Price
    price_ax.plot(df['price'])
    price_ax.set_title("{} price".format(data['symbol'].upper()))

    # Technical Indicator outputs
    indicator_outputs_subplot(ta_ax, df, data['indicator'])

    # Buy/sell signals
    signals_subplot(sig_ax, df)

    plt.savefig("{}_{}_signal".format(data['symbol'], data['indicator']))


def plot_signals(data: dict):
    # Convert to dataframe with datetime index
    df = pd.DataFrame.from_dict(data['map'], orient='index')
    df.index = pd.to_datetime(df.index)

    if data['indicator'] == 'BollingerBands':
        plot_superimposed_indicator(data, df)
    else:
        plot_separate_indicator(data, df)
    
####

if __name__ == "__main__":
    json_str = ''
    for line in fileinput.input():
        json_str += line

    try:
        data = json.loads(json_str)
    except Exception as e:
        print("Exception decoding JSON: {}".format(e))

    plot_signals(data)

