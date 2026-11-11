use crate::cfg::Cfg;
use crate::encoder::Encoder;

use eframe::egui::{self, Modifiers};
use egui::{Color32, FontFamily, FontId}; // Style
use std::cell::Cell;

use std::path::PathBuf;

use rfd;

use crate::note::{Note, note_events};


const MINIMAL_UI: bool = true;

pub fn open_file_dialog(_ctx: &egui::Context) -> Option<PathBuf> {
    rfd::FileDialog::new().pick_file()
}

pub fn save_file_dialog(_ctx: &egui::Context) -> Option<PathBuf> {
    rfd::FileDialog::new().save_file()
}

pub fn extended_ui(note:&mut Note, ui: &mut egui::Ui, ctx: &egui::Context) {
    if !MINIMAL_UI {
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

use std::sync::atomic::{AtomicBool, Ordering};
impl<'a> eframe::App for Note<'a> {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        eprintln!("[cannot_get_this_to_work] {}", self.log);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font = FontId::new(self.cfg.fontsize, FontFamily::Monospace);
        let color = Color32::from_rgb(28, 28, 28);
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(6 as f32)))
            .show(ctx, |ui| {
            ui.heading("");
            let output = egui::ScrollArea::vertical()
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
                        let keypress_crtl = Modifiers { ctrl: true,
                                    alt: false,
                                    mac_cmd: false,
                                    shift: false,
                                    command: false,
                                };
                        let keypress_shift_crtl = Modifiers { ctrl: true,
                                    alt: false,
                                    mac_cmd: false,
                                    shift: true,
                                    command: false,
                                };

                        if e.consume_key(keypress_shift_crtl, egui::Key::S){
                            note_events::save_optional_backup_file(self, ||save_file_dialog(ctx));
                        } 
                        if e.consume_key(keypress_crtl, egui::Key::S){
                            note_events::savefile(self, ||save_file_dialog(ctx));
                        } 
                        if e.consume_key(keypress_crtl, egui::Key::E){
                            note_events::clear_log(self);
                        }

                        

                    });
                });
                // resize frame for the textboxes
                static FITTED: AtomicBool = AtomicBool::new(false);
                if !FITTED.swap(true, Ordering::Relaxed) {
                    let width = ctx.screen_rect().width();
                    let height = (output.content_size.y + 4 as f32 * self.cfg.fontsize).ceil();
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(width, height)));
                }
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
