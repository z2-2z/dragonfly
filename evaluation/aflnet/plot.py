#!/usr/bin/env python3

import sys
import json
import matplotlib.pyplot as plt

def extract(data, idx):
    x = []
    y = []
    
    for elem in data:
        x.append(float(elem[0]))
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
    
    x, y = extract(data, 4) # coverage
    
    fig, ax = plt.subplots()
    ax.plot(x, y)
    #ax.set_xscale("log")
    plt.show()

if __name__ == "__main__":
    main()
