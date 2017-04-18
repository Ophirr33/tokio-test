#!/usr/bin/env python3
import sys
import socket
import struct
import time


def ltob(l):
    return struct.pack('!Q', l)


def btol(b):
    if len(b) != 8:
        print(b)
    return struct.unpack('!Q', b)[0]


if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("./test-it.py ip_addr port")
        sys.exit(1)
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((sys.argv[1], int(sys.argv[2])))
    # The first loop works
    for i in range(1001):
        s.send(ltob(i))
        resp = s.recv(8)
    print(btol(resp)) # Should print 2000
    # The second loop breaks with error 30 or 54
    try:
        for i in range(1002):
            s.send(ltob(i))
            resp = s.recv(8)
        print(btol(resp)) # Should print 2002
    except OSError as e:
        print("Failed to send 1002 values due to {}".format(e))
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((sys.argv[1], int(sys.argv[2])))
    # But then it works with 1001 values again
    for i in range(1000, 2001):
        s.send(ltob(i))
        resp = s.recv(8)
    print(btol(resp)) # should print 4000
