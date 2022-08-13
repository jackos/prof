# Prof
Run commands to profile your Rust program

## Valgrind
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

