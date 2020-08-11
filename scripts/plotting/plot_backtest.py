# Output objects are usually like this, with some exceptions:
# TODO:

import fileinput
import json
import matplotlib.pyplot as plt
import datetime as dt
import pandas as pd
import jsonschema

# TODO: chart security and indicator in subplots
# TODO: buy/sell points as vertical lines
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
    colors = df['signal'].map(
        lambda elem: 'red' if elem < 0 else 'green').values
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
    fig.suptitle("{} {} signals".format(
        data['symbol'].upper(), data['indicator']))

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
    fig.suptitle("{} {} signals".format(
        data['symbol'].upper(), data['indicator']))

    # Price
    price_ax.plot(df['price'])
    price_ax.set_title("{} price".format(data['symbol'].upper()))

    # Technical Indicator outputs
    indicator_outputs_subplot(ta_ax, df, data['indicator'])

    # Buy/sell signals
    signals_subplot(sig_ax, df)

    plt.savefig("{}_{}_signal".format(data['symbol'], data['indicator']))


def plot_backtest(data: dict):
    trades = {}
    for k, v in data['trades']['trades'].items():
        val = ''
        if v == "Out" or v == "Hold":
            val = v
        else:
            val = list(v.keys())[0]
        
        trades[k] = val

    # Convert to dataframe with datetime index
    structure = {
        "trades": trades,
        "daily_portvals": data['performance']['daily_portvals'],
        "price": data['prices']['map']
        # "daily_returns" : data['performance']['daily_returns']
    }
    df = pd.DataFrame.from_dict(structure,)
    df.index = pd.to_datetime(df.index)

    sd = df.index[0]
    ed = df.index[-1]

    plt.figure()
    fig, (price_ax) = plt.subplots(1, sharex=True)
    fig.suptitle("{} performance on {} from {} to {}".format(
        data['model_name'], data['symbol'].upper(), sd.strftime("%Y-%m-%d"), ed.strftime("%Y-%m-%d")))

    # Normalized Price
    norm_prices = df['price'] / df['price'].iloc[0]
    price_ax.plot(norm_prices, label="{} price".format(data['symbol'].upper()))
    # ax.set_title("{} price".format(data['symbol'].upper()))

    # Normalized daily portvals
    price_ax.plot(df['daily_portvals'] / df['daily_portvals'].iloc[0], label="Portfolio value")

    # Buy/sells
    price_xmin, price_xmax, price_ymin, price_ymax = plt.gca().axis()
    for date, trade in df['trades'].iteritems():
        if trade != 'Long' and trade != 'Short':
            continue

        if trade == 'Long':
            color = 'green'
            ymin = price_ymin
            ymax = norm_prices[date]
        elif trade == "Short":
            color = 'red'
            ymin = price_ymin
            ymax = norm_prices[date]

        price_ax.vlines(date, color=color, linestyle="-", ymin=ymin, ymax=ymax)

    plt.legend()

    plt.savefig("{}_{}_backtest".format(data['symbol'], data['model_name']))

####


if __name__ == "__main__":
    # json_str=''
    # for line in fileinput.input():
    #     json_str += line

    # print(json_str[0:40])
    import sys
    import codecs
    # import fileinput

    # handle = codecs.getreader("utf_") open(sys.argv[0]) if len(sys.argv) > 0 else sys.stdin
    handle = open(sys.argv[1]) if len(sys.argv) > 1 else sys.stdin
    
    try:
        data=json.load(handle)
    except Exception as e:
        print("Exception decoding JSON: {}".format(e))

    plot_backtest(data)
