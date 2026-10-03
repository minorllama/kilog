# KiloG

A barebones GUI texteditor that's just a `egui` wrapper. Initially started a GUI version of [kilo](https://github.com/antirez/kilo), [kilo-rs]([https://github.com/arminha/kilo-rs).
Now intended to be hackable.
Build with: 
```bash
 CTIME="$(date '+%m-%d-%y-%H.%M.%S')" cargo +stable build
./target/debug/gbuf  -h
            $ gbuf -gbuf:afile -spec:z 
            $ gunzip -c afile # the saved file is gzipped
```

It writes files after gzipping, and gunzips before read, all easy to hack. The `-spec` option is how the file io is [encoded](https://github.com/minorllama/kilog/blob/main/src/encoder.rs).
