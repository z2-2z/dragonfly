#!/bin/bash
set -e;
test -d /ftproot;
test -d /output;
chmod -R 777 /output;
nohup ./fuzzer fuzz --output /output > /output/output &
