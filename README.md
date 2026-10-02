# GBuf

A hackable barebones GUI texteditor that's just a `egui` wrapper. Build and try with: 
```bash
 CTIME="$(date '+%m-%d-%y-%H.%M.%S')" cargo +stable build
./target/debug/gbuf  -h
            $ gbuf -gbuf:afile
            $ gunzip -c afile # the saved file is gzipped
```

It writes files after gzipping, and gunzips before read, all easy to hack. 
