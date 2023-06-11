#!/usr/bin/env python3

import sys
import json
import matplotlib.pyplot as plt

def extract(data, key):
    x = []
    y = []
    
    if key == "exec_sec":
        data = data[1:]

    for elem in data:
        if key == "exec_sec":
            x.append(elem["run_time"]["secs"])
        else:
            x.append(elem["executions"])
        y.append(elem[key])
        
    return x, y

def main():
    logfile = sys.argv[1]
    
    data = []
    with open(logfile) as f:
        for line in f:
            line = json.loads(line.strip())
            data.append(line)
    
    #x, y = plot_data = extract(data, "exec_sec")
    x, y = extract(data, "corpus")
    
    fig, ax = plt.subplots()
    ax.plot(x, y)
    #ax.set_xscale("log")
    plt.show()

if __name__ == "__main__":
    main()
