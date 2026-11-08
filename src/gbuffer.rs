use crate::cfg::Cfg;
use crate::encoder::Encoder;
use crate::utils;

use eframe::egui::{self, Modifiers};
use egui::{Color32, FontFamily, FontId}; // Style
use std::cell::Cell;

use std::path::PathBuf;

use rfd;

use crate::note::{Note, BackUpType};


const MINIMAL_UI: bool = true;

pub fn open_file_dialog(_ctx: &egui::Context) -> Option<PathBuf> {
    rfd::FileDialog::new().pick_file()
}

pub fn save_file_dialog(_ctx: &egui::Context) -> Option<PathBuf> {
    rfd::FileDialog::new().save_file()
}


pub fn extended_ui(note:&mut Note, ui: &mut egui::Ui, ctx: &egui::Context) {
    if !MINIMAL_UI {
        //let sp_button = egui::Button::new("sp").min_size((&[40.0, 1.0]).into());
        let open_button = egui::Button::new("open").min_size((&[40.0, 1.0]).into());
        if ui.add(open_button).clicked() {
            if let Some(path) = open_file_dialog(ctx) {
                note.filepath = Some(path.clone());
                let status = note.loadfile();
                note.log_string(&status);
            }
        }
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
                            .font(font.clone()) //TextStyle::Monospace)
                            .background_color(color.clone())
                            .lock_focus(true)
                            .text_color(egui::Color32::GRAY)
                            .desired_width(f32::INFINITY)
                            .desired_rows(self.cfg.n_rows),
                    );
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add(
                        egui::TextEdit::multiline(&mut self.log)
                            .font(font) //TextStyle::Monospace)
                            .background_color(color.clone())
                            .lock_focus(false)
                            .text_color(egui::Color32::GRAY)
                            .desired_width(f32::INFINITY)
                            .desired_rows(1)
                            .interactive(false), 
                    );
                    let save_button = egui::Button::new("save").min_size((&[40.0, 1.0]).into());


                    
                    extended_ui(self, ui, ctx);

                    if ui.add(save_button).clicked() {
                        if !self.filepath.is_some() {
                            self.filepath = save_file_dialog(ctx);
                        }
                        let status = self.savefile();
                        self.log_string(&status);
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
                            // handle "ctrl-shift-s";
                            if self.filepath.is_some() {
                                let filename = self.filepath.as_ref().expect("..");
                                let filename = filename.to_str().unwrap();
                                let status = &mut "".to_owned();
                                /*self.backing(
                                    filename,
                                    status,
                                    BackUpType::BeforeWrite,
                                    &utils::timetag(),
                                );*/
                                BackUpType::BeforeWrite.backing(
                                    filename,
                                    status,
                                    self,
                                    &utils::timetag(),
                                );
                                self.log_string(&status);
                                let status = self.savefile();
                                self.log_string(&status)
                            } else {
                                self.log_string(
                                    &save_file_dialog(ctx)
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
                                self.log_string(&status);
                            } else {
                                save_file_dialog(ctx);
                            }
                        };
                    });
                });
        });
    }
}

pub fn ui(_f: Option<&String>, e: Note) {
    
    fn with_context<'a >(_cc: &eframe::CreationContext<'_>, e: Note<'a>) -> Note<'a> {
        e
    }

    let _ = eframe::run_native(
        "",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(with_context(cc, e)))),
    );
}

pub fn main(c: &Cfg, encoder: Box<dyn Encoder>) {
    let target = c.keys.get("-gbuf");
    let mut buffer = "".to_owned();
    let signal = Cell::new(false);
    let note = Note::new(encoder, &c, &mut buffer, &signal);
    ui(target, note);
}
