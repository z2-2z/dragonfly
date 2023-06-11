#!/usr/bin/env python3

import sys
import json
import matplotlib.pyplot as plt

# total_execs, unix_time, cycles_done, cur_path, paths_total, pending_total, pending_favs, map_size, unique_crashes, unique_hangs, max_depth, execs_per_sec
TOTAL_EXECS = 0
UNIX_TIME = 1
PATHS_TOTAL = 4
EXECS_PER_SEC = 11

def extract(data, idx):
    x = []
    y = []
    start_time = float(data[0][UNIX_TIME])
    
    for elem in data:
        if idx == EXECS_PER_SEC:
            x.append(float(elem[UNIX_TIME]) - start_time)
        else:
            x.append(float(elem[TOTAL_EXECS]))
        y.append(float(elem[idx]))
    
    return x, y

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
    
    x, y = extract(data, PATHS_TOTAL)
    #x, y = extract(data, EXECS_PER_SEC)
    
    fig, ax = plt.subplots()
    ax.plot(x, y)
    #ax.set_xscale("log")
    plt.show()

if __name__ == "__main__":
    main()
