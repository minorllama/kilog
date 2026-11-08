use crate::cfg::Cfg;
use crate::encoder::Encoder;
use crate::utils;

use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::{Display, Formatter};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub enum BackUpType {
    OnOpen,
    AfterWrite,
    OnExit,
    BeforeWrite,
}
impl Display for BackUpType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BackUpType::OnOpen => write!(f, "open"),
            BackUpType::BeforeWrite => write!(f, "bw"),
            BackUpType::AfterWrite => write!(f, "aw"),
            BackUpType::OnExit => write!(f, "exit"),
        }
    }
}

impl BackUpType {
    pub fn backing(self, filename: &str, m: &mut String, note:&Note, now: &String) {
        fn logbackup(tag: BackUpType, filename: &str, now: &String) -> String {
            if let Ok(backed) = utils::backfile(filename, &format!("{}.{}", tag, now)) {
                format!(":backed[{tag}]:{backed}")
            } else {
                format!(":failed_backing[{tag}]")
            }
        }
        match self {
            BackUpType::OnExit|BackUpType::OnOpen|BackUpType::BeforeWrite => {
                if note.cfg.trace { m.push_str(&logbackup(self, filename, now)) } 
            }
            BackUpType::AfterWrite => {
                if note.cfg.history { m.push_str(&logbackup(self, filename, now))} 
            }
        };
    }
}


#[derive(Debug)]
pub struct NoteCfg {
    trace: bool,
    history: bool,
    pub fontsize: f32,
    on_exit: bool,
    pub n_rows: usize,
    exit_overwrite: bool,
}

pub struct Note<'a> {
    txt: String,
    pub log: String,
    logbuffer: VecDeque<String>,
    pub filepath: Option<PathBuf>,
    coding: Box<dyn Encoder>,
    pub buffer: &'a mut String,
    signal: &'a Cell<bool>,
    pub cfg: NoteCfg,
}

impl<'a> Note<'a> {
    pub fn state(&self) -> &String {
        &self.txt
    }

    pub fn new(
        encoder: Box<dyn Encoder>,
        cfg: &Cfg,
        buffer: &'a mut String,
        sig: &'a Cell<bool>,
    ) -> Self {
        let mut note = Note {
            signal: sig,
            txt: "".to_owned(),
            logbuffer: VecDeque::new(),
            log: "".to_owned(),
            filepath: cfg.get("-gbuf").map(|x| x.into()),
            coding: encoder,
            buffer: buffer,
            cfg: NoteCfg {
                n_rows: cfg
                    .get("-rows")
                    .unwrap_or("18".to_string())
                    .parse()
                    .unwrap(),
                trace: cfg.has("-trace"),
                on_exit: cfg.has("-onexit"),
                history: cfg.has("-history"),
                fontsize: cfg
                    .get("-fontsize")
                    .unwrap_or("14.0".to_string())
                    .parse()
                    .unwrap(),
                exit_overwrite: cfg.has("-overwrite"),
            },
        };
        if note.filepath.is_some() {
            let log = note.loadfile();
            note.log_string(&log);
        }
        eprintln!("{:?}", note.cfg);
        note
    }

    pub fn logger<T: Display>(&mut self, m: &T) {
        eprintln!("{}", m);
    }
    pub fn log_string(&mut self, m: &str) {
        self.log.clear();
        self.log.push_str(m);
        self.logger(&m);
        self.logbuffer.push_back(self.log.clone());
    }



    pub fn savefile(&self) -> String {
        let mut data = self.buffer.as_bytes().to_vec();
        if let Some(filepath) = self.filepath.as_ref() {
            self.coding.encode(&mut data);
            let filename = filepath.to_str();
            let now = &utils::timetag();
            let filename2 = filename.unwrap();
            let mut m = if let Err(e) = utils::to_file(filename2, &data) {
                format!("{now} failed @ save[{filename2}] with:{e}")
            } else {
                let mut hasher = DefaultHasher::new();
                self.buffer.hash(&mut hasher);
                let hash_value = hasher.finish();
                format!("{now} {hash_value} save[{filename2}]")
            };
            BackUpType::AfterWrite.backing(filename2, &mut m, self, now);  
            m

        } else {
            "no_vald_filepath".to_owned()
        }
    }

    pub fn loadfile(&mut self) -> String {
        let path = self.filepath.as_ref().unwrap().to_str().unwrap();
        match utils::from_file(path) {
            Ok(mut data) => {
                self.coding.decode(&mut data);
                self.buffer.push_str(&String::from_utf8(data).unwrap());
                let now = &utils::timetag();
                let mut m = format!("{now} opened:[{path}]");
                BackUpType::OnOpen.backing(path, &mut m, self, now); 
                m
            }
            Err(e) => e.to_string(),
        }
    }





    fn handle_exit(&self, data: &Vec<u8>) {
        let tag = utils::timetag();
        if let Some(filepath) = &self.filepath {
            let fname = &filepath.display().to_string(); 
            let dropfile = &format!("{}.drop.{}", &fname, &tag);
            let backfile = if self.cfg.exit_overwrite {
                fname
            } else {
                dropfile
            };
            let write_status = utils::to_file(&backfile, &data);
            eprintln!("{:?}:{}", &write_status,&backfile);
            if !write_status.is_ok(){
                eprintln!("[!!!!write_failed:{}]", &backfile);
            }
            if !self.cfg.exit_overwrite {
                let newpath = format!("{}.mvdrp.{}", fname, &tag);
                let move1 = fs::rename(&fname, &newpath);
                let move2 = fs::rename(&backfile, fname);
                eprintln!(
                    "# cleanup[{}]: {:?} {:?} [{},{}] ",
                    fname, move1, move2, newpath, backfile
                );
            }
        }
    }
}


impl<'a> Drop for Note<'a> {
    fn drop(&mut self) {
        eprintln!("#############");
        eprintln!("# {}", self.logbuffer.len());
        for e in &self.logbuffer {
            eprintln!("# {}", e);
        }
        eprintln!("#############");

        if self.cfg.on_exit {
            let mut data: Vec<u8> = self
                .buffer
                .as_bytes()
                .into_iter()
                .map(|x| x.clone())
                .collect();
            self.coding.encode(&mut data);
            self.handle_exit(&data)
        } else {
            eprintln!("#notrace");
        }
        self.signal.replace(true);
    }
}