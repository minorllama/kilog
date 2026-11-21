use crate::cfg::Cfg;
use crate::encoder::Encoder;


use eframe::egui::{self, Modifiers};
use eframe::egui::{text::LayoutJob, Color32, TextFormat, Galley};
use egui::{FontFamily, FontId}; // Style

use std::cell::Cell;
use std::path::PathBuf;
use std::sync::Arc;


use std::collections::HashMap;

use rfd;

use crate::rich::note as note;
use crate::rich::note::{Note, note_events, NoteState};



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
                let status = note_events::loadfile(note);
                note.log_string(&status);
            }
        }
    }
}



fn make_overlay_layout(font: FontId, text_color: Color32, state:&HashMap<NoteState, String>) 
-> impl FnMut(&egui::Ui, &str, f32) -> Arc<Galley> + use<> // need use<> so the lifetime for state does not get attached to return type
{
    let highlight = state.get(&NoteState::HighLight).map(|x|x.clone());

    move |ui, text, wrap_width| {
        let normal = TextFormat {
            font_id: font.clone(),
            color: text_color,
            ..Default::default()
        };
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width;
        if let Some(word) = highlight.as_deref() && let Some((before, after)) = text.split_once(word) {
            let highlighted = TextFormat {
                background: Color32::YELLOW,
                ..normal.clone()
            };
            job.append(before, 0.0, normal.clone());
            job.append(word, 0.0, highlighted);
            job.append(after, 0.0, normal);
        } else {
            job.append(text, 0.0, normal);
        }
        ui.fonts(|fonts| fonts.layout_job(job))
    }
}

impl<'a>  Note<'a> {
    fn make_textbox<'b, 'c>(&'c mut self, box_type:u32, font:FontId, text_color:Color32, background_color:Color32) ->  egui::TextEdit<'b> 
            where 'c:'b {

        let buffer:&mut String = match box_type{ 
            0 => self.buffer,
            1 => &mut self.log,
            _ => &mut self.console,
        };
        let rows = if box_type == 0 { self.cfg.n_rows } else { 1 };  
        egui::TextEdit::multiline(buffer)
            .font(font.clone()) //TextStyle::Monospace)
            .background_color(background_color)
            .lock_focus(true)
            .text_color(text_color)
            .desired_width(f32::INFINITY)
            .desired_rows(rows)
            .interactive(box_type == 0 || box_type == 2)
            .lock_focus(box_type == 0)
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
impl<'a> eframe::App for Note<'a> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font = FontId::new(self.cfg.fontsize, FontFamily::Monospace);
        let background_color = Color32::from_rgb(28, 28, 28);
        let background_color2 = Color32::from_rgb(20, 20, 20);
        let text_color = egui::Color32::GRAY;

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
                    const TEXTBOX:u32 = 0;
                    const LOGBOX:u32 = 1;
                    const CONSOLEBOX:u32 = 2;
                    
                    let save_button = egui::Button::new("save").min_size((&[40.0, 1.0]).into());

                    let mut layouter = make_overlay_layout(font.clone(), text_color.clone(), &self.state);
                    let textbox = self.make_textbox(TEXTBOX, font.clone(), text_color.clone(), background_color.clone());
                    
                    //ui.add(textbox.layouter(&mut layouter)); 
                    ui.add(textbox); 
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add(self.make_textbox(LOGBOX, font.clone(), text_color.clone(), background_color.clone()));
                    ui.separator();
                    ui.separator();
                    ui.add(self.make_textbox(CONSOLEBOX, font.clone(), text_color.clone(), background_color2.clone()));

                    extended_ui(self, ui, ctx);

                    if ui.add(save_button).clicked() {
                        note_events::save_optional_backup_file(self, ||save_file_dialog(ctx));
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
                            note_events::savefile(self, ||save_file_dialog(ctx), true);
                        } 
                        if e.consume_key(keypress_crtl, egui::Key::E){
                            note_events::clear_log(self);
                            note_events::clear_console(self);
                        }
                        if e.consume_key(keypress_crtl, egui::Key::F){
                            note_events::find_highlight(self); 
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







