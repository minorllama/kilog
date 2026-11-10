use kilog::cfg::Cfg;
use kilog::encoder::Encoder;

#[cfg(feature = "encoder_ext")]
use kilog::encoder_ext::StackedEncoder2;

#[cfg(not(feature = "encoder_ext"))]
use kilog::encoder::StackedEncoder as StackedEncoder2;

use kilog::gbuffer;
use kilog::utils;

use std::{env, fs};

const CTIME: &'static str = match option_env!("CTIME") {
    Some(value) => value,
    None => "[unknown]",
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let cfg = Cfg::from_vec(&args);
   

    if cfg.has("-gbuf") {
        let encoder = StackedEncoder2::from_str_spec(cfg.keys.get("-spec").unwrap_or(&"".to_string())).unwrap();
        gbuffer::main(&cfg, Box::new(encoder))
    } else if cfg.has("-pipe") {
        let decoder = StackedEncoder2::from_str_spec(cfg.keys.get("-r").unwrap_or(&"".to_string())).unwrap();
        let encoder = StackedEncoder2::from_str_spec(cfg.keys.get("-w").unwrap_or(&"".to_string())).unwrap();
        let infile = cfg.keys.get("-pipe").unwrap();
        let data = {
            let mut buffer: Vec<u8> = fs::read(infile).unwrap();
            decoder.decode(&mut buffer);
            encoder.encode(&mut buffer);
            buffer
        };
        if let Some(outfile) = cfg.keys.get("-out") {
            let write_status = utils::to_file(outfile, &data);
            if !write_status.is_ok() {
                eprintln!("[write_failed:{}]", outfile);
            }
        } else {
            let text = String::from_utf8(data).unwrap();
            println!("{}", text);
        }
    } else if cfg.has("-bytes") {
        for e in &cfg.values {
            println!("{:?} {:?}", e, e.as_bytes().to_vec())
        }
    } else {
        println!("compiled:{}", CTIME);
        let usage = "
            ./target/release/kilog -gbuf:1.z -spec:z 
            
            # file opened is gzipped as specified by -spec and all changes are saved gzipped, see src/encoder.rs";
        if cfg.has("-h") {
            println!("{}", usage);
        }
    }
}
