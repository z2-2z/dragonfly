#!/usr/bin/env python3

from pwn import *

def resp():
    global r
    line = r.recvline()
    status = line[:3]
    rem = line[3:]
    status = int(status.decode("ascii"))
    rem = rem.decode("utf-8").strip()
    return (status, rem)

def cmd(s):
    global r
    r.sendline(s.encode("utf-8") + b"\r")

with remote("localhost", 2121, level="debug") as r:
    assert(resp()[0] == 220)

    cmd("USER ftp")
    assert(resp()[0] == 331)

    cmd("PASS x")
    assert(resp()[0] == 230)

    cmd("CWD uploads")
    assert(resp()[0] == 250)

    cmd("EPSV")
    status, arg = resp()
    assert(status == 229)
    port = int(arg.split("|")[3])
    data = remote("localhost", port)

    cmd("STOR abc.txt")
    assert(resp()[0] == 150)

    data.send(b"y")

    cmd("ABOR")
    
    data.sendline(b"yeehaw")

    # if ABOR comes after sending data the server
    # treats the STOR as successfully completed with 226

    data.close()

    assert(resp()[0] == 426)
    assert(resp()[0] == 226)

    cmd("QUIT")
    assert(resp()[0] == 221)
