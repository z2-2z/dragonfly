# Overhead

## Baseline
On a single core:
```
[Stats #0] run time: 0h-2m-0s, clients: 1, corpus: 1, objectives: 0, executions: 723502, exec/sec: 6.027k
```
On 8 cores:
```
[Stats #7] run time: 0h-2m-0s, clients: 11, corpus: 10, objectives: 0, executions: 2580040, exec/sec: 21.46k
```

## Simple Server
On a single core:
```
[Stats #0] run time: 0h-2m-0s, clients: 1, corpus: 9, objectives: 0, executions: 722925, exec/sec: 6.019k
```
On 8 cores:
```
[Stats #10] run time: 0h-2m-0s, clients: 11, corpus: 78, objectives: 0, executions: 2471953, exec/sec: 20.56k
```

## Summary
- Packet processing and delivery has negligible overhead. the test binaries probably bottleneck on forkserver stuff
- scalability factor: 3.33 / 4 (85%) which is ok based on the fact that the binary spends ~60% of its execution time in the kernel according to htop
