#!/bin/bash
set -ex;
test -d /ftproot;
test -d /output;
chmod -R 777 /output /ftproot;
timeout 24h ./fuzzer fuzz --output /output $* &> /output/output
