#!/bin/bash
set -ex;
test -d /ftproot;
test -d /output;
chmod -R 777 /output;
timeout 24h ./fuzzer fuzz --output /output $* &> /output/output
