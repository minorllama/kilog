use crate::cfg::Cfg;
use crate::encoder::Encoder;
use crate::utils;
//use eframe::egui::TextStyle;
use eframe::egui::{self, Modifiers};
use egui::{Color32, FontFamily, FontId}; // Style
use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::{Display, Formatter};

use std::fs;
use std::path::PathBuf;

use rfd;

const MINIMAL_UI: bool = true;
//const SCRATCHFILE:&'static str = "scratchfile";

#[derive(Debug)]
enum BackUpType {
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

#[derive(Debug)]
pub struct NoteCfg {
    trace: bool,
    history: bool,
    fontsize: f32,
    on_exit: bool,
    n_rows: usize,
    exit_overwrite: bool,
}

pub struct Note<'a> {
    txt: String,
    log: String,
    logbuffer: VecDeque<String>,
    filepath: Option<PathBuf>,
    coding: Box<dyn Encoder>,
    buffer: &'a mut String,
    signal: &'a Cell<bool>,
    cfg: NoteCfg,
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
            eprintln!("{}", log);
        }
        eprintln!("{:?}", note.cfg);
        note
    }

    pub fn logger<T: Display>(&mut self, m: &T) {
        eprintln!("{}", m);
    }
    pub fn log_string(&mut self, m: &str) {
        self.log.push('\n');
        self.log.push_str(m);
    }
    pub fn log_err(&mut self, m: &str) {
        self.log.push('\n');
        self.log.push_str(m);
        println!("\n{}", m);
    }

    fn with_context(_cc: &eframe::CreationContext<'_>, e: Note<'a>) -> Self {
        e
    }

    fn backing(&self, filename: &str, m: &mut String, tag: BackUpType, now: &String) {
        fn logbackup(filename: &str, m: &mut String, tag: BackUpType, now: &String) {
            if let Ok(backed) = utils::backfile(filename, &format!("{}.{}", tag, now)) {
                m.push_str(&format!("\n  -backed[{tag}]:{backed}"));
            } else {
                m.push_str(&format!("\n  -failed_backing[{tag}]"));
            };
        }
        match tag {
            BackUpType::OnExit => {
                if self.cfg.trace {
                    logbackup(filename, m, tag, now);
                }
            }
            BackUpType::OnOpen => {
                if self.cfg.trace {
                    logbackup(filename, m, tag, now);
                }
            }
            BackUpType::BeforeWrite => {
                if self.cfg.trace {
                    logbackup(filename, m, tag, now);
                }
            }
            BackUpType::AfterWrite => {
                if self.cfg.history {
                    logbackup(filename, m, tag, now);
                }
            }
        }
    }

    fn savefile(&self) -> String {
        let mut data = self.buffer.as_bytes().to_vec();
        let mut m = String::new();
        if let Some(filepath) = self.filepath.as_ref() {
            self.coding.encode(&mut data);
            let filename = filepath.to_str();
            let now = &utils::timetag();
            let filename2 = filename.unwrap();
            if let Err(e) = utils::to_file(filename2, &data) {
                m.push_str(&format!("{now} failed @ save[{filename2}] with:{e}"));
            } else {
                m.push_str(&format!("{now} save[{filename2}]"));
            };
            self.backing(filename2, &mut m, BackUpType::AfterWrite, now);
        } else {
            m.push_str("no_vald_filepath");
        }
        m
    }

    fn loadfile(&mut self) -> String {
        let path = self.filepath.as_ref().unwrap().to_str().unwrap();
        let mut m = String::new();
        let _log = match utils::from_file(path) {
            Ok(mut data) => {
                self.coding.decode(&mut data);
                self.buffer.push_str(&String::from_utf8(data).unwrap());
                let now = &utils::timetag();
                m.push_str(&format!("{now} opened:[{path}]"));
                self.backing(path, &mut m, BackUpType::OnOpen, now)
            }
            Err(e) => m.push_str(&e.to_string()),
        };
        m
    }

    fn open_file_dialog(&mut self, _ctx: &egui::Context) -> Option<PathBuf> {
        rfd::FileDialog::new().pick_file()
    }

    fn save_file_dialog(&self, _ctx: &egui::Context) -> Option<PathBuf> {
        rfd::FileDialog::new().save_file()
    }

    fn extended_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if !MINIMAL_UI {
            //let sp_button = egui::Button::new("sp").min_size((&[40.0, 1.0]).into());
            let open_button = egui::Button::new("open").min_size((&[40.0, 1.0]).into());
            if ui.add(open_button).clicked() {
                if let Some(path) = self.open_file_dialog(ctx) {
                    self.filepath = Some(path.clone());
                    let status = self.loadfile();
                    self.log_string(&status);
                }
            }
        }
    }

    fn handle_exit(&self, data: &Vec<u8>) {
        let tag = utils::timetag();
        if let Some(filepath) = &self.filepath {
            let fname = &filepath.display().to_string(); //&filename(&self.filepath);
            let dropfile = &format!("{}.drop.{}", &fname, &tag);
            let backfile = if self.cfg.exit_overwrite {
                fname
            } else {
                dropfile
            };
            eprintln!(
                "{:?}:{}",
                utils::to_file(&backfile, &data),
                //utils::write_byteslice(Some(&backfile).as_ref(), &data),
                &backfile
            );
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
        let mut data: Vec<u8> = self
            .buffer
            .as_bytes()
            .into_iter()
            .map(|x| x.clone())
            .collect();
        self.coding.encode(&mut data);
        if self.cfg.on_exit {
            self.handle_exit(&data)
        } else {
            eprintln!("#notrace");
        }
        self.signal.replace(true);
    }
}

impl<'a> eframe::App for Note<'a> {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        eprintln!("[cannot_get_this_to_work] {}", self.log);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //let mut style = ctx.style().clone();
        //access and modify the "Body" text style
        //style.text_styles.insert(TextStyle::Body, FontId::new(18.0, egui::FontFamily::Proportional));
        let font = FontId::new(self.cfg.fontsize, FontFamily::Monospace);
        //pub const fn from_rgb(r: u8, g: u8, b: u8).
        let color = Color32::from_rgb(28, 28, 28);
        //ctx.set_style(style);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("");
            egui::ScrollArea::vertical()
                .id_salt("serial_output")
                .auto_shrink([true; 2])
                .stick_to_bottom(true)
                .enable_scrolling(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(self.buffer)
                            .font(font) //TextStyle::Monospace)
                            .background_color(color.clone())
                            .lock_focus(true)
                            .text_color(egui::Color32::GRAY)
                            .desired_width(f32::INFINITY)
                            .desired_rows(self.cfg.n_rows),
                    );
                    let save_button = egui::Button::new("save").min_size((&[40.0, 1.0]).into());

                    self.extended_ui(ui, ctx);

                    if ui.add(save_button).clicked() {
                        if !self.filepath.is_some() {
                            self.filepath = self.save_file_dialog(ctx);
                        }
                        let status = self.savefile();
                        self.logger(&status);
                    }
                    ui.label(format!(""));
                    let _input = &ctx.input_mut(|e| {
                        if e.consume_key(
                            Modifiers {
                                ctrl: true,
                                alt: false,
                                mac_cmd: false,
                                shift: true,
                                command: false,
                            },
                            egui::Key::S,
                        ) {
                            //eprintln!("ctrl-shift-s");
                            if self.filepath.is_some() {
                                let filename = self.filepath.as_ref().expect("..");
                                let filename = filename.to_str().unwrap();
                                let status = &mut "".to_owned();
                                self.backing(
                                    filename,
                                    status,
                                    BackUpType::BeforeWrite,
                                    &utils::timetag(),
                                );
                                self.logger(&status);
                                let status = self.savefile();
                                self.logger(&status)
                            } else {
                                self.logger(
                                    &self
                                        .save_file_dialog(ctx)
                                        .expect("...")
                                        .to_str()
                                        .expect("..."),
                                )
                            };
                        };
                        if e.consume_key(
                            Modifiers {
                                ctrl: true,
                                alt: false,
                                mac_cmd: false,
                                shift: false,
                                command: false,
                            },
                            egui::Key::S,
                        ) {
                            if self.filepath.is_some() {
                                let status = self.savefile();
                                self.logger(&status);
                                self.log = status;
                            } else {
                                self.save_file_dialog(ctx);
                            }
                        };
                    });
                });
        });
    }
}

pub fn ui(_f: Option<&String>, e: Note) {
    let _ = eframe::run_native(
        "",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(Note::with_context(cc, e)))),
    );
}

pub fn main(c: &Cfg, encoder: Box<dyn Encoder>) {
    let target = c.keys.get("-gbuf");
    let mut buffer = "".to_owned();
    let signal = Cell::new(false);
    let note = Note::new(encoder, &c, &mut buffer, &signal);
    ui(target, note);
}
