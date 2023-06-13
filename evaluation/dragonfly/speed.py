#!/usr/bin/env python3

import sys
import json
import statistics

def main():
    logfile = sys.argv[1]
    
    data = []
    with open(logfile) as f:
        for line in f:
            line = json.loads(line.strip())
            data.append(line)
    
    execs = list(
        map(
            lambda x: x["exec_sec"],
            data
        )
    )
    left = min(execs)
    mean = statistics.fmean(execs)
    right = max(execs)
    print(f"min={left}, mean={mean}, max={right}")

if __name__ == "__main__":
    main()
