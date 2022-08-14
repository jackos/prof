# Prof
This is in a very early prototyping stage, there are a lot of great tools that can profile Rust programs from the C / C++ ecosystem.

The problem is the commands are difficult to remember, this tool aims to remedy that and provide outputs like JSON that can be piped to other tools like UI's.

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
