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
YAML output with bytes as text (human-readable)
```bash
cargo prof heap
```
```yaml
allocated_total: 2KB 119B
frees: 11
allocations: 11
allocated_at_exit: 0B
blocks_at_exit: 0
```
JSON outputs with total bytes
```bash
cargo prof heap
```
```json
{"allocated_total":2167,"frees":11,"allocations":11,"allocated_at_exit":0,"blocks_at_exit":0}
```

### Standalone
Runs on any binary

Install
```bash
cargo install prof
```

Profile `ripgrep` if `rg` is on your `path`
```
prof leak --bin rg
```

Pass `ripgrep` some arguments to stress it and see if any memory leaks 
```bash
prof leak --bin rg -- a -uuu
```
```yaml
definitely_lost: 0B
indirectly_lost: 0B
possibly_lost: 528B
still_reachable: 369KB 218B
supressed: 0B
definitely_lost_blocks: 0
indrectly_lost_blocks: 0
possibly_lost_blocks: 9
still_reachable_blocks: 89
supressed_blocks: 0
```

Give `grep` a try as well and notice some data is leaked, even on a single file: 
```bash
prof leak --bin grep -- a Cargo.toml 
```
```yaml
definitely_lost: 272B
indirectly_lost: 352B
possibly_lost: 128B
still_reachable: 109KB 490B
supressed: 0B
definitely_lost_blocks: 272
indrectly_lost_blocks: 11
possibly_lost_blocks: 1
still_reachable_blocks: 15
supressed_blocks: 0
```
