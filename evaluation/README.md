# Evaluation

To evaluate dragonfly's performance, it is measured how much coverage
it can achieve in comparison to AFLNet.
The program under test is [ProFTPD](http://proftpd.org/) and both fuzzers start with an empty corpus and a dictionary of supported
FTP commands.
The campaign is run for 24h and after that, the _statement_ coverage, obtained via gcov, is compared.

## Dragonfly Fuzzer
```
cd dragonfly
docker build --pull -t evaluation-dragonfly .
mkdir output
docker run -v "$PWD/output":/output evaluation-dragonfly
```

The container must be stopped with `docker stop <container-id>`, Ctrl+C will not work.

## AFLNet Fuzzer
```
cd aflnet
docker build --pull -t evaluation-aflnet .
mkdir output
docker run -v "$PWD/output":/output evaluation-aflnet
```

The container must be stopped with `docker stop <container-id>`, Ctrl+C will not work.
