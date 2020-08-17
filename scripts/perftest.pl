# Quick and dirty script for an end-to-end run of performance for the ML model.

use strict;
use warnings;

my $symbol = $ARGV[0] or die 'provide symbol';
my $secret = $ARGV[1] or die 'provide secret';
my $end_train_date = $ARGV[2] or die 'provide end train date';
my $start_test_date = $ARGV[3] or die 'provide start test date';
my $data = $ARGV[4] or 0;

unless ($data) {
  system("cargo run -- -d AlphaVantage --secret $secret $symbol print -i rsi > $symbol\_rsi.json")
  == 0  or die;
  $data = "$symbol\_rsi.json";
}

system("cargo run -- -d $symbol\_rsi.json --end-date $end_train_date $symbol train -s MACD")
 == 0  or die;

system("cargo run -- -d $symbol\_rsi.json --start-date $start_test_date $symbol back-test MachineLearningModel -m $symbol.bin 10000 > $symbol\_perf.json")
 == 0  or die;

print "Plotting...\n";
system("python ./scripts/plotting/plot_backtest.py $symbol\_perf.json")
 == 0  or die;
