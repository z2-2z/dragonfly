#!/usr/bin/env python3

import sys
import statistics

# total_execs, unix_time, cycles_done, cur_path, paths_total, pending_total, pending_favs, map_size, unique_crashes, unique_hangs, max_depth, execs_per_sec
EXECS_PER_SEC = 11

def main():
    logfile = sys.argv[1]
    
    data = []
    with open(logfile) as f:
        for line in f:
            line = line.strip()
            
            if line.startswith("#"):
                continue
            
            data.append(
                tuple(
                    map(str.strip, line.split(","))
                )
            )
    
    execs = list(
        map(
            lambda x: float(x[EXECS_PER_SEC]),
            data
        )
    )
    mean = statistics.fmean(execs)
    left = min(execs)
    right = max(execs)
    print(f"min={left}, mean={mean}, max={right}")
    

if __name__ == "__main__":
    main()
