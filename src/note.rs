use crate::cfg::Cfg;
use crate::encoder::Encoder;
use crate::utils;

use std::cell::Cell;
use std::collections::VecDeque;

use std::fmt::{self, Display, Formatter};
use std::io::{self, BufReader, Read};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::fs;
use std::path::PathBuf;



#[derive(Debug)]
pub enum SaveFile {
    IgnoredEmpty,
    IgnoredUnchanged,
    Ok(String),
    Err(String),
}


impl SaveFile {
    fn as_str<'a >(&'a self) -> &'a str {
        match self {
            SaveFile::IgnoredEmpty => "[ignored_empty_buffer]",
            SaveFile::IgnoredUnchanged => "[ignored_unchanged_buffer]",
            SaveFile::Ok(log) => log,
            SaveFile::Err(log) => log,
        }
    }
}

impl From<&SaveFile> for String {
    fn from(save_ok: &SaveFile) -> Self {
        save_ok.as_str().to_owned()
    }
}


impl From<SaveFile> for String {
    fn from(save_ok: SaveFile) -> Self {
        save_ok.as_str().to_owned()
    }
}



impl Display for SaveFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", String::from(self))
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
    buffer0: String,
    pub log: String,
    logbuffer: VecDeque<String>,
    pub filepath: Option<PathBuf>,
    coding: Box<dyn Encoder>,
    pub buffer: &'a mut String,
    signal: &'a Cell<bool>,
    pub cfg: NoteCfg,
}

impl<'a> Note<'a> {
    pub fn last_state(&self) -> &String {
        &self.buffer0
    }

    pub fn is_buffer_dirty(&self) -> bool {
        self.buffer0 != *self.buffer
    }

    pub fn new(
        encoder: Box<dyn Encoder>,
        cfg: &Cfg,
        buffer: &'a mut String,
        sig: &'a Cell<bool>,
    ) -> Self {
        let mut note = Note {
            signal: sig,
            buffer0: "".to_owned(),
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
            let log = note_events::loadfile(&mut note);
            note.log_string(&log);
        } else if cfg.has("-stdin") {
            let mut data = Vec::new();
            let now = &utils::timetag();
            if let Ok(_) = utils::from_stdin(&mut data) {
                note.coding.decode(&mut data);
                note.buffer.push_str(&String::from_utf8(data).unwrap());
                note.log_string(&format!("{now} read:[stdin]"));
            } else {
                note.log_string(&format!("{now} read_failed:[stdin]"));
            }
        };
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


    pub fn savefile_checked(&mut self) -> SaveFile {
        eprintln!("{:?}", self.is_buffer_dirty());
        if !self.is_buffer_dirty() {
            SaveFile::IgnoredUnchanged 
        } else {
            self.buffer0.clone_from(self.buffer);
            self.savefile()
        }
    }


    pub fn savefile(&self) -> SaveFile {
        let mut data = self.buffer.as_bytes().to_vec();
        if let Some(filepath) = self.filepath.as_ref() {
            if let Some(filename2) = filepath.to_str() {
                let now = &utils::timetag();
                self.coding.encode(&mut data);
                let mut m = if let Err(e) = utils::to_file_atomic(filename2, &data) {
                    format!("{now} failed @ save[{filename2}] with:{e}")
                } else {
                    let mut hasher = DefaultHasher::new();
                    self.buffer.hash(&mut hasher);
                    let hash_value = hasher.finish();
                    format!("{now} {hash_value:016x} save[{filename2}]")
                };
                SaveFile::Ok(m)
            } else {
                SaveFile::Err(format!("malformed_filename:{:?}", filepath))
            }
        } else {
            SaveFile::Err("no_vald_filepath".to_owned())
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
            let write_status = utils::to_file_atomic(&backfile, &data);
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
        let sepline = "#############";
        eprintln!("{}", sepline);
        eprintln!("# {}", self.logbuffer.len());
        for e in &self.logbuffer {
            eprintln!("# {}", e);
        }
        eprintln!("{}", sepline);

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


pub mod note_events {

    use std::fmt::{self, Display, Formatter};
    use std::path::PathBuf;

    use crate::utils;
    use super::{Note, SaveFile};



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

    pub fn savefile<F>(note: &mut Note, get_filename: F, checked:bool)
        where F: FnOnce() -> Option<PathBuf> {
        if !note.filepath.is_some() {
            if let Some(newname) = get_filename(){
                note.filepath = Some(newname);
            }
        }
        if !note.filepath.is_some(){
            note.log_string("[save_cancelled]");
        } else {
            let status = if checked {
                note.savefile_checked()
            } else {
                note.savefile()
            };
            note.log_string(status.as_str());
        }
    }

    pub fn save_optional_backup_file<F>(note:&mut  Note, get_filename: F)
        where F: FnOnce() -> Option<PathBuf> {
        if !note.filepath.is_some(){
            if let Some(newname) = get_filename(){
                note.filepath = Some(newname);
            }
        }
        if !note.filepath.is_some(){
            note.log_string("[save_cancelled]");
        } else {
            let filename = note.filepath.as_ref().expect("..");
            let filename = filename.to_str().unwrap();
            let status = &mut "".to_owned();
            backup(BackUpType::BeforeWrite,
                filename,
                status,
                note,
                &utils::timetag(),
            );
            note.log_string(&status);
            let status = note.savefile();
            note.log_string(status.as_str());

        }
    }
    pub fn clear_log(note:&mut  Note){
        note.log_string("");
    }

    pub fn loadfile(note:&mut Note) -> String {
        let path = note.filepath.as_ref().unwrap().to_str().unwrap();
        match utils::from_file(path) {
            Ok(mut data) => {
                note.coding.decode(&mut data);
                note.buffer.push_str(&String::from_utf8(data).unwrap());
                note.buffer0.clone_from(note.buffer);
                let now = &utils::timetag();
                let mut m = format!("{now} opened:[{path}]");
                backup(BackUpType::OnOpen, path, &mut m, note, now); 
                m
            }
            Err(e) => e.to_string(),
        }
    }

    pub fn backup(backup:BackUpType, filename: &str, m: &mut String, note:&Note, now: &String) {
        fn logbackup(tag: BackUpType, filename: &str, now: &String) -> String {
            if let Ok(backed) = utils::backfile(filename, &format!("{}.{}", tag, now)) {
                format!(":backed[{tag}]:{backed}")
            } else {
                format!(":failed_savefile[{tag}]")
            }
        }
        match backup {
            BackUpType::OnExit|BackUpType::OnOpen|BackUpType::BeforeWrite => {
                if note.cfg.trace { m.push_str(&logbackup(backup, filename, now)) } 
            }
            BackUpType::AfterWrite => {
                if note.cfg.history { m.push_str(&logbackup(backup, filename, now))} 
            }
        };
    }


}