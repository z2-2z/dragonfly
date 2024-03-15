#!/usr/bin/env python3

import sys
import json
import matplotlib.pyplot as plt

def max_corpus_size(data):
    max_size = 0
    
    for client in data["clients"]:
        max_size = max(max_size, client["corpus_size"])
        
    return max_size

def main(logfiles):
    fig, ax = plt.subplots()
    
    for logfile in logfiles:
        num_clients = 0
        x = []
        y = []
        
        with open(logfile) as f:
            for line in f:
                data = json.loads(line.strip())
                x.append(data["run_time"]["secs"])
                y.append(max_corpus_size(data))
                num_clients = max(len(data["clients"]), num_clients)
        
        ax.plot(x, y, label=f"{logfile} ({num_clients} cores)")
    
    ax.set_xlabel("time")
    ax.set_ylabel("max. corpus size")
    ax.set_title("dragonfly fuzzers")
    ax.grid(True)
    ax.legend()
    
    plt.show()

if __name__ == "__main__":
    main(sys.argv[1:])
