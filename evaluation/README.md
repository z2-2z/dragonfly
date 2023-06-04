# Evaluation

To evaluate dragonfly's performance, it is measured how much coverage
it can achieve in comparison to AFLNet.
The program under test is [ProFTPD](http://proftpd.org/).
Both fuzzers start out with an empty corpus and a dictionary of supported
FTP commands and fuzz for 24h.

## Dragonfly Fuzzer
```
cd dragonfly
docker build --pull -t evaluation-dragonfly .
mkdir output
docker run -v "$PWD/output":/output evaluation-dragonfly
```

The container must be stopped with `docker stop`, Ctrl+C will not work.

## AFLNet Fuzzer
TODO
