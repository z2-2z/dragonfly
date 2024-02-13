#!/usr/bin/env python3

BLACKLIST = [
    ' ', '\t', '\n', chr(0x0b), chr(0x0c), '\r',
    '+', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
]

result = []

for i in range(256):
    if i >= 128 or chr(i) in BLACKLIST:
        result.append(0)
    else:
        result.append(1)
        
print(result)
