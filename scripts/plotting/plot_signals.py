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

json_str = ''
for line in fileinput.input():
  json_str += line

try:
  data = json.loads(json_str)
except e:
  print("Exception decoding JSON: {}".format(e))

# plt.figure()
# plt.xlabel("Date")
# plt.ylabel("Date")