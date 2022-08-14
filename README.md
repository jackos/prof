# Prof
This is in a very early prototyping stage, there are a lot of great tools that can profile Rust programs from the C / C++ ecosystem.

I always forget the commands to run and have to look them up in my notes, and tools like Valgrind don't have great outputs for piping to other programs, this will make it a lot easier to run the commands, and pipe the results to other tools like UI's.


Run commands to profile your Rust program

## Quickstart

There are two versions, one will run on your cargo targets:

Install
```bash
cargo install cargo-prof
``` 
JSON output with bytes as int's
```
cargo prof heap
```
YAML output with bytes as text (human-readable)
```
cargo prof heap -h
```

And the other will run on any binary:

Install
```bash
cargo install prof
```
JSON output with bytes as int's
```
prof heap --bin mybin
```
YAML output with bytes as text (human-readable)
```
prof heap --bin mybin -h
```
