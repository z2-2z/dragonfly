#!/bin/bash
set -e

if [[ -z "$1" ]];
then
    echo "Usage: $0 <fuzz target>"
    exit 1
fi

taskset -c 0 cargo fuzz run "$1" -- -only_ascii=1 -reduce_inputs=0 -use_cmp=0
