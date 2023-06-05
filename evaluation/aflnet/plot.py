#!/usr/bin/env python3

import sys
import json
import matplotlib.pyplot as plt

def extract(data, idx):
    x = []
    y = []
    start_time = float(data[0][0])
    
    for elem in data:
        x.append(float(elem[0]) - start_time)
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
    
    #x, y = plot_data = extract(data, 10) # exec/s
    x, y = extract(data, 3) # coverage
    
    fig, ax = plt.subplots()
    ax.plot(x, y)
    #ax.set_xscale("log")
    plt.show()

if __name__ == "__main__":
    main()
