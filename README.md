# KiloG

A barebones GUI texteditor that's just a `egui` wrapper. Initially started a GUI version of [kilo](https://github.com/antirez/kilo), [kilo-rs](https://github.com/arminha/kilo-rs).
Now intended to be hackable.
Build with: 
```bash
 CTIME="$(date '+%m-%d-%y-%H.%M.%S')" cargo build --release
./target/debug/kilog -h
            ./target/release/kilog -gbuf:1.z -spec:z 
            
            # file opened is gzipped as specified by -spec and all changes are saved gzipped, see src/encoder.rs
```


