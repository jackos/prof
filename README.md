# Prof
This is in a very early prototyping stage. 

There are a lot of great tools that can profile Rust programs from the C / C++ ecosystem, this aims to wrap them in an easy-to-use CLI that will eventually be cross-platform, providing a human-readable output as well as `json` that can be piped to other tools such as UI's.

## Quickstart

There are two versions:

### Cargo
Runs on your Cargo targets

Install
```bash
cargo install cargo-prof
``` 
JSON output with total bytes in int format
```
cargo prof heap
```
YAML output with bytes as text (human-readable)
```
cargo prof heap -h
```

### Standalone
Runs on any binary

Install
```bash
cargo install prof
```
JSON output with total bytes in int format
```
prof heap --bin mybin
```
YAML output with bytes as text (human-readable)
```
prof heap --bin mybin -h
```
