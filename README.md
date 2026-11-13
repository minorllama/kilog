# KiloG

A minimal GUI texteditor that's a wrapper around an `egui` frame. Initially started a GUI version of [kilo](https://github.com/antirez/kilo), [kilo-rs](https://github.com/arminha/kilo-rs).
Now intended to be hackable.
Build with: 
```bash
 CTIME="$(date '+%m-%d-%y-%H.%M.%S')" cargo build --release
./target/release/kilog -h
            ./target/release/kilog -gbuf:1.z -spec:z             
```

The compile time is saved in the binary. The `-spec:z` specifies input/output encoder for the file, see [encoder.rs](src/encoder.rs): `z` specifies that the file content is compressed. 

