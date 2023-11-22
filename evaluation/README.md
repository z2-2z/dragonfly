# Evaluation

To evaluate dragonfly's performance, it is measured how much coverage
it can achieve in comparison to AFLNet.
The program under test is [ProFTPD](http://proftpd.org/) and both fuzzers start with an empty corpus and a dictionary of supported
FTP commands.
The campaign is run for 24h and after that, the _statement_ coverage, obtained via gcov, is compared.

## Dragonfly
```
cd dragonfly
docker build --pull -t evaluation-dragonfly -f Dockerfile-fuzz .
mkdir output
docker run --security-opt seccomp:unconfined -v "$PWD/output":/output evaluation-dragonfly
```

The container must be stopped with `docker stop <container-id>`, Ctrl+C will not work.

To collect the coverage report execute:
```
cd dragonfly
docker build --pull -t coverage-dragonfly -f Dockerfile-cov .
docker run -v "$PWD/output":/output coverage-dragonfly
```

The report can be found in `output/report/index.html`.

## AFLNet
```
cd aflnet
docker build --pull -t evaluation-aflnet -f Dockerfile-fuzz .
mkdir output
echo core | sudo tee /proc/sys/kernel/core_pattern
pushd /sys/devices/system/cpu
echo performance | sudo tee cpu*/cpufreq/scaling_governor
popd
docker run --security-opt seccomp:unconfined -v "$PWD/output":/output evaluation-aflnet
```

The container must be stopped with `docker stop <container-id>`, Ctrl+C will not work.

To collect the coverage report execute:
```
cd aflnet
docker build --pull -t coverage-aflnet -f Dockerfile-cov .
docker run -v "$PWD/output":/output coverage-aflnet
```

The report can be found in `output/report/index.html`.

## Results
- AFLNet:
    - avg. exec/s: 20
    - coverage: 10.7%
    - bugs: 0
- Dragonfly without state feedback:
    - avg. exec/s: 80
    - coverage: 31.1%
    - bugs: TODO (> 0)
- Dragonfly + feedback about valid cmds:
    - avg. exec/s: 170
    - coverage: 31.4%
    - bugs: TODO (> 0)

In all scenarios above the fuzzers were not able to synthesize a valid login sequence inspite of having
access to a valid USER and PASS command.
Try again with a valid input in corpus:

