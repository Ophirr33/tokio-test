```
cargo run --release&
python3 test-it.py 0.0.0.0 12345
```
Should print:
```
2000
Failed to send 1002 values due to (Differing errors here)
4000
```

Trying to figure out why the tcp connection gets messed up on sending more than 1001 values
