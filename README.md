# Prof
Run commands to profile your Rust program

## Quickstart

There are two versions, one will run on your cargo targets:
```bash
cargo install cargo-prof
cargo prof valgrind bytes
cargo prof valgrind bytes --bin other_bin
```
And the other will run on any binary:
```bash
cargo install prof
prof valgrind bytes --bin mybin
```

