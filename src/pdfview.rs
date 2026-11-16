use std::collections::HashMap;

use eframe::egui;
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings};

pub fn pdfview(bytes: Vec<u8>) -> eframe::Result {
    eframe::run_native(
        "PDFView",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Ok(Box::new(PdfViewer {
                pending_bytes: Some(bytes),
                ..Default::default()
            }))
        }),
    )
}

#[derive(Default)]
struct PdfViewer {
    pending_bytes: Option<Vec<u8>>,
    pdf: Option<Pdf>,
    textures: HashMap<usize, egui::TextureHandle>,
    error: Option<String>,
    page_index: usize,
    texture: Option<egui::TextureHandle>,
}

impl PdfViewer {
    fn render(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(bytes) = self.pending_bytes.take() {
            match Pdf::new(bytes) {
                Ok(pdf) => self.pdf = Some(pdf),
                Err(err) => self.error = Some(format!("Could not open PDF: {err:?}")),
            }
        }

        if let Some(error) = &self.error {
            ui.label(error);
            return;
        }

        if let Some(pdf) = &self.pdf {
            let pages = pdf.pages();
            let total = pages.len();

            if total == 0 {
                ui.label("This PDF has no pages.");
                return;
            }

            let (back, forward, scroll_y) = ui.input(|input| {
                let scroll_y = if input.key_pressed(egui::Key::ArrowUp) {
                        64.0
                    } else if input.key_pressed(egui::Key::ArrowDown) {
                        -64.0
                    } else {
                        0.0
                    };
                (   input.key_pressed(egui::Key::ArrowLeft),
                    input.key_pressed(egui::Key::ArrowRight),
                    scroll_y)
            });

            let next_page = match (back, forward) {
                (true, false) => self.page_index.saturating_sub(1),
                (false, true) => (self.page_index + 1).min(total - 1),
                _ => self.page_index,
            };

            if next_page != self.page_index {
                self.page_index = next_page;
                self.texture = None;
            }

            ui.label(format!("Page {}/{}", self.page_index + 1, total));

            let texture = self.texture.get_or_insert_with(|| {
                let page = &pages[next_page];
                let (width, height) = page.render_dimensions();
                let scale = 1600.0 / width.max(height);

                let pixels = hayro::render(
                    page,
                    &RenderCache::new(),
                    &InterpreterSettings::default(),
                    &RenderSettings::default(),
                    &PixmapSettings {
                        x_scale: scale,
                        y_scale: scale,
                        bg_color: WHITE,
                    },
                );

                let image = egui::ColorImage::from_rgba_premultiplied(
                    [pixels.width() as usize, pixels.height() as usize],
                    pixels.data_as_u8_slice(),
                );

                ui.ctx().load_texture("PDF page", image, egui::TextureOptions::LINEAR)
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                if scroll_y != 0.0 {
                    ui.scroll_with_delta(egui::vec2(0.0, scroll_y));
                }

                let width = ui.available_width();
                if width > 0.0 {
                    let page_size = texture.size_vec2();
                    let size = page_size * (width / page_size.x);
                    ui.add(egui::Image::new((texture.id(), size)).fit_to_exact_size(size));
                }
            });
        }
    }


 

}

impl eframe::App for PdfViewer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render(ui, frame);
        });
    }
}