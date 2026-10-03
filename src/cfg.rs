use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;

#[derive(Debug)]
pub enum LogMsg {
    Msg(String),
    TaggedMsg(String, String),
}

#[derive(Debug)]
pub struct Cfg {
    pub flags: HashSet<String>,
    pub values: Vec<String>,
    pub keys: HashMap<String, String>,
    pub sys: Vec<String>,
    pub log: RefCell<Vec<LogMsg>>,
    should_log: bool,
    rt_warn: bool,
}

impl Cfg {
    fn new() -> Cfg {
        Cfg {
            flags: HashSet::new(),
            values: Vec::new(),
            keys: HashMap::new(),
            sys: Vec::new(),
            log: RefCell::new(Vec::new()),
            should_log: false,
            rt_warn: false,
        }
    }
    pub fn get(self: &Cfg, k: &str) -> Option<String> {
        self.keys.get(k).map(|x| x.clone())
    }

    pub fn has(self: &Cfg, key: &str) -> bool {
        self.flags.contains(key) || self.keys.contains_key(key)
    }
    pub fn logger<T: Borrow<str> + Display + ?Sized>(self: &Cfg, m: &T) {
        self.log_entry(LogMsg::Msg(m.to_string()));
    }
    pub fn get_opt<'a, 'b, 'c: 'a>(&'a self, k: &'b str, default: &'c String) -> &'a String {
        if let Some(value) = self.keys.get(k) {
            value
        } else {
            default
        }
    }
    pub fn log_tagged<T: Borrow<str> + Display + ?Sized>(self: &Cfg, tag: &T, m: &T) {
        self.log_entry(LogMsg::TaggedMsg(tag.to_string(), m.to_string()));
    }
    pub fn log_entry(&self, m: LogMsg) {
        if self.rt_warn {
            {
                eprintln!("#__log:{:?}", &m);
            }
        }
        self.log.borrow_mut().push(m);
    }
    pub fn log_all(&self) {
        for e in self.log.borrow().iter() {
            eprintln!("#__log:{:?}", e);
        }
    }
    pub fn from_vec(vec: &Vec<String>) -> Self {
        let key_delimit = ':';
        let mut cfg = Cfg::new();
        let mut fields: Vec<&str> = Vec::new();
        for v in vec.iter() {
            cfg.sys.push(v.to_string());
            if v.chars().next().unwrap() == '-' {
                fields.clear();
                fields = v.split(key_delimit).map(|s| s).collect();
                let parsed_ok: bool = match fields.len() {
                    1 => cfg.flags.insert(v.to_string()),
                    2 => cfg
                        .keys
                        .insert(fields[0].to_string(), fields[1].to_string())
                        .is_some(),
                    _ => {
                        cfg.log_tagged("malformed", v);
                        false
                    }
                };
                cfg.should_log = cfg.should_log || !parsed_ok;
            } else {
                cfg.values.push(v.to_string());
            }
        }
        cfg.should_log = cfg.should_log || cfg.has("-verbose");
        if cfg.has("-rt_warn") {
            cfg.rt_warn = true;
            cfg.log_tagged("rt_warn[on]", "true");
            cfg.log_all();
        }
        cfg
    }
}

impl Drop for Cfg {
    fn drop(&mut self) {
        if self.should_log && !self.rt_warn {
            self.log_all();
        }
    }
}
