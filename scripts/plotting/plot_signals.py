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

# TODO: differentiate by indicator
# TODO: security symbol in title
# TODO: chart security and indicator in subplots
# TODO: buy/sell signals beyond some threshold as vertical lines
# TODO: schema validation of input https://python-jsonschema.readthedocs.io/en/stable/validate/

json_str = ''
for line in fileinput.input():
  json_str += line

try:
  data = json.loads(json_str)
except Exception as e:
  print("Exception decoding JSON: {}".format(e))

df = pd.DataFrame.from_dict(data['map'], orient='index')
df.index = pd.to_datetime(df.index)

plt.figure()
plt.xlabel("Date")
plt.ylabel("RSI")

plt.plot(df)

plt.savefig('test')