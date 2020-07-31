# TODO: ingest JSON, plot

# Output objects are usually like this, with some exceptions:
# {
#   "symbol"  : string
#   "dates"   : [array of date strings]
#   "outputs" : [array of floats],
#   "prices"  : [array of floats],
#   "signals" : [array of floats]
# }

import fileinput
import json
import matplotlib.pyplot as plt
import datetime as dt
import pandas as pd

# TODO: differentiate by indicator
# TODO: security symbol in title
# TODO: chart security and indicator in subplots
# TODO: buy/sell signals beyond some threshold as vertical lines

json_str = ''
for line in fileinput.input():
  json_str += line

try:
  data = json.loads(json_str)
except e:
  print("Exception decoding JSON: {}".format(e))

outputs = list(map(lambda a: a[0], data['outputs']['outputs']))
prices = data['prices']
dates = list(map(lambda d: dt.datetime.strptime(d, "%Y-%m-%d"), data['dates']))
df = pd.DataFrame(data={'price':prices, 'rsi': outputs}, index=dates)

plt.figure()
plt.xlabel("Date")
plt.ylabel("RSI")

plt.plot(df)

plt.savefig('test')