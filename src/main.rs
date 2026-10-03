use kilog::cfg::Cfg;
use kilog::encoder::Encoder;
use kilog::encoder::*;
use kilog::gbuffer;
use std::{env, fs};

const CTIME: &'static str = match option_env!("CTIME") {
    Some(value) => value,
    None => "...",
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let cfg = Cfg::from_vec(&args);

    let encoder = StackedEncoder::from_str_spec(cfg.keys.get("-spec").unwrap_or(&"".to_string())).unwrap();

    if cfg.has("-gbuf") {
        gbuffer::main(&cfg, Box::new(encoder))
    } else if cfg.has("-pipe") {
        for f in &cfg.values[1..] {
            let mut data: Vec<u8> = fs::read(f).unwrap();
            encoder.decode(&mut data);
            let text = String::from_utf8(data).unwrap();
            println!("{}", text);
        }
    } else {
        println!("compiled:{}", CTIME);
        let usage = "  $ kilog -gbuf:afile";
        if cfg.has("-h") {
            println!("{}", usage);
        }
    }
}
