# Prof
Run commands to profile your Rust program

## Quickstart
```bash
cargo install cargo-prof
cargo prof
```

## Valgrind
Linux only, won't work on windows 

Get the total bytes allocated to the heap by your Rust binary
```bash
cargo prof valgrind bytes
```
Target a different binary
```bash
cargo prof valgrind bytes mybinary
```
Change the default bytes that are subtracted
```bash
cargo prof valgrind bytes mybinary
```

