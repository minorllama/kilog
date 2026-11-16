use eframe::egui;
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::{Pdf, page::Page};
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
    page_index: usize,
    texture: Option<(usize, egui::TextureHandle)>,
    error: Option<String>,
    copy_status: Option<String>,
}



impl PdfViewer {
    fn render(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.open_pending_pdf();
        if let Some(error) = &self.error {
            ui.label(error);
            return;
        }
        if let Some(total) = self.pdf.as_ref().map(|pdf| pdf.pages().len()) {
            if total == 0 {
                ui.label("empty_pdf");
                return;
            }
            self.update_page(ui, total);
            self.show_page(ui, total);
        };
    }

    fn open_pending_pdf(&mut self) {
        let Some(bytes) = self.pending_bytes.take() else {
            return;
        };
        match Pdf::new(bytes) {
            Ok(pdf) => self.pdf = Some(pdf),
            Err(err) => self.error = Some(format!("pdf_open_fail:{err:?}")),
        }
    }

    fn update_page(&mut self, ui: &egui::Ui, total: usize) {
        let (back, forward) = ui.input(|input| {
            (
                input.key_pressed(egui::Key::ArrowLeft),
                input.key_pressed(egui::Key::ArrowRight),
            )
        });

        let next = match (back, forward) {
            (true, false) => self.page_index.saturating_sub(1),
            (false, true) => (self.page_index + 1).min(total - 1),
            _ => self.page_index,
        };

        if next != self.page_index {
            self.page_index = next;
            self.texture = None;
            self.copy_status = None;
        }
    }

    fn show_page(&mut self, ui: &mut egui::Ui, total: usize) {
        let Some(pdf) = &self.pdf else { return };
        let page_number = self.page_index + 1;

        ui.horizontal(|ui| {
            ui.label(format!("page:{page_number}/{total}"));
            if ui.button("copy_text").clicked() {
                self.copy_status = Some(copy_page_text(ui, pdf, page_number));
            }
        });

        if let Some(status) = &self.copy_status {
            ui.label(status);
        }

        let page = &pdf.pages()[self.page_index];
        egui::ScrollArea::vertical().show(ui, |ui| {
            scroll_arrows(ui);
            draw_page(ui, page, &mut self.texture);
        });
    }
}

fn scroll_arrows(ui: &egui::Ui) {
    ui.input(|input| {
        let up = input.key_pressed(egui::Key::ArrowUp);
        let down = input.key_pressed(egui::Key::ArrowDown);
        let delta = 64.0;
        if up {
            ui.scroll_with_delta(egui::vec2(0.0, delta));
        } else if down {
            ui.scroll_with_delta(egui::vec2(0.0, -delta));
        } 
    });
}

fn draw_page<'a>(ui: &mut egui::Ui, page: &'a Page<'a>, texture: &mut Option<(usize, egui::TextureHandle)>){
    let width = ui.available_width();
    let (page_width, page_height) = page.render_dimensions();
    if !(width <= 0.0 || page_width <= 0.0 || page_height <= 0.0){
        let display_scale = width / page_width;
        let size = egui::vec2(width, page_height * display_scale);
        let long_edge = page_width.max(page_height);
        let render_side = target_render_side(ui, long_edge * display_scale);

        if texture.as_ref().map(|(side, _)| *side) != Some(render_side) {
            let scale = render_side as f32 / long_edge;
            *texture = Some((render_side, render_page_texture(ui, page, scale)));
        }

        if let Some((_, handle)) = texture {
            ui.add(egui::Image::new((handle.id(), size)).fit_to_exact_size(size));
        }
    }
}

fn target_render_side(ui: &egui::Ui, displayed_long_edge: f32) -> usize {
    let physical_pixels = displayed_long_edge * ui.ctx().pixels_per_point();
    let limit = ui.input(|input| input.max_texture_side).clamp(1, 4096);

    // reuse the texture across small changes in the window width.
    let rounded_pixels = (physical_pixels / 128.0).ceil() * 128.0;
    (rounded_pixels as usize).clamp(1, limit)
}

fn render_page_texture<'a>(ui: &egui::Ui, page: &'a Page<'a>, scale: f32) -> egui::TextureHandle {
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

    ui.ctx().load_texture("pdf_page", image, egui::TextureOptions::LINEAR)
}

fn copy_page_text(ui: &egui::Ui, pdf: &Pdf, page_number: usize) -> String {
    let text = lopdf::Document::load_mem(pdf.data().as_ref())
        .and_then(|doc| doc.extract_text(&[page_number as u32]));

    match text {
        Ok(text) if text.trim().is_empty() => "no_available_text.".into(),
        Ok(text) => {
            ui.ctx().copy_text(text);
            format!("[copied_text:page={page_number}]")
        }
        Err(err) => format!("failed_text_extract:{err}"),
    }
}

impl eframe::App for PdfViewer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render(ui, frame);
        });
    }
}