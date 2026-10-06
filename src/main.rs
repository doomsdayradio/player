#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod audio;
mod spectrum;

use audio::{Player, Status};
use eframe::egui::{self, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use egui_phosphor::regular;
use spectrum::{Spectrum, BANDS};
use std::time::{Duration, Instant};

const BACKGROUND: Color32 = Color32::from_rgb(43, 33, 24);
const SURFACE: Color32 = Color32::from_rgb(28, 20, 16);
const TEXT: Color32 = Color32::from_rgb(232, 226, 214);
const MUTED: Color32 = Color32::from_rgb(173, 145, 120);
const AMBER: Color32 = Color32::from_rgb(255, 157, 47);
const GLOW: Color32 = Color32::from_rgb(255, 209, 102);
const RUST: Color32 = Color32::from_rgb(180, 75, 47);
const SIGNAL: Color32 = Color32::from_rgb(131, 255, 171);
const PHOSPHOR: Color32 = Color32::from_rgb(209, 255, 69);
const LINE: Color32 = Color32::from_rgb(83, 54, 38);
const LOGO: &[u8] = include_bytes!("../assets/doomsday-radio.png");
const RADIO_FONT: &[u8] = include_bytes!("../assets/ShareTechMono-Regular.ttf");

fn main() -> eframe::Result {
    let logo = image::load_from_memory(LOGO)
        .expect("Embedded station logo must be a valid PNG")
        .into_rgba8();
    let icon = image::imageops::resize(&logo, 32, 32, image::imageops::FilterType::Triangle);
    let logo = egui::ColorImage::from_rgba_unmultiplied(
        [logo.width() as usize, logo.height() as usize],
        logo.as_raw(),
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Doomsday Radio")
            .with_inner_size([440.0, 400.0])
            .with_min_inner_size([360.0, 360.0])
            .with_icon(egui::IconData {
                rgba: icon.into_raw(),
                width: 32,
                height: 32,
            }),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Doomsday Radio",
        options,
        Box::new(move |context| Ok(Box::new(RadioApp::new(context, logo)))),
    )
}

struct RadioApp {
    player: Player,
    spectrum: Spectrum,
    logo: egui::TextureHandle,
    volume: f32,
    muted: bool,
    last_frame: Instant,
    started: Option<Instant>,
}

impl RadioApp {
    fn new(context: &eframe::CreationContext<'_>, logo: egui::ColorImage) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "radio-mono".into(),
            egui::FontData::from_static(RADIO_FONT).into(),
        );
        for family in [egui::FontFamily::Monospace, egui::FontFamily::Proportional] {
            fonts
                .families
                .get_mut(&family)
                .unwrap()
                .insert(0, "radio-mono".into());
        }
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        context.egui_ctx.set_fonts(fonts);
        let logo =
            context
                .egui_ctx
                .load_texture("doomsday-logo", logo, egui::TextureOptions::LINEAR);
        let mut style = (*context.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = BACKGROUND;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.selection.bg_fill = AMBER;
        style.visuals.widgets.inactive.bg_fill = SURFACE;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, LINE);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, MUTED);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(63, 44, 28);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, AMBER);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, GLOW);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(78, 58, 43);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, AMBER);
        style.spacing.item_spacing = Vec2::new(10.0, 6.0);
        context.egui_ctx.set_style(style);
        Self {
            player: Player::default(),
            spectrum: Spectrum::new(),
            logo,
            volume: 0.65,
            muted: false,
            last_frame: Instant::now(),
            started: None,
        }
    }

    fn toggle(&mut self) {
        if self.player.session.is_some() {
            self.player.stop();
            self.started = None;
        } else {
            self.player
                .start(if self.muted { 0.0 } else { self.volume });
            self.started = Some(Instant::now());
        }
    }

    fn visualizer(&self, ui: &mut egui::Ui, height: f32) {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let painter = ui.painter_at(rect);
        for row in 1..4 {
            let vertical = rect.top() + height * row as f32 / 4.0;
            painter.line_segment(
                [
                    Pos2::new(rect.left(), vertical),
                    Pos2::new(rect.right(), vertical),
                ],
                Stroke::new(1.0_f32, LINE.gamma_multiply(0.6)),
            );
        }
        for column in 0..=8 {
            let horizontal = rect.left() + rect.width() * column as f32 / 8.0;
            painter.line_segment(
                [
                    Pos2::new(horizontal, rect.top()),
                    Pos2::new(horizontal, rect.top() + 4.0),
                ],
                Stroke::new(1.0_f32, LINE),
            );
        }
        let step = rect.width() / BANDS as f32;
        let segments = ((height - 4.0) / 5.0).floor().max(1.0) as usize;
        for (index, level) in self.spectrum.levels.iter().enumerate() {
            let left = rect.left() + index as f32 * step;
            let lit = (level * segments as f32).ceil() as usize;
            for segment in 0..lit.max(1) {
                let bottom = rect.bottom() - segment as f32 * 5.0;
                let bar = Rect::from_min_max(
                    Pos2::new(left, bottom - 3.0),
                    Pos2::new(left + (step - 3.0).max(2.0), bottom),
                );
                let position = segment as f32 / segments as f32;
                let color = if *level < 0.01 {
                    LINE
                } else if position < 0.32 {
                    RUST
                } else if position < 0.74 {
                    PHOSPHOR
                } else {
                    TEXT
                };
                painter.rect_filled(bar, 0.0, color);
            }
        }
    }

    fn station_header(&self, ui: &mut egui::Ui) {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 144.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let center = rect.center();
        for tick in 0..48 {
            let angle = std::f32::consts::TAU * tick as f32 / 48.0;
            let direction = Vec2::new(angle.cos(), angle.sin());
            painter.line_segment(
                [center + direction * 68.0, center + direction * 72.0],
                Stroke::new(1.0_f32, LINE),
            );
        }
        let image_height = 140.0;
        let image_width = image_height * self.logo.size()[0] as f32 / self.logo.size()[1] as f32;
        let image_rect = Rect::from_center_size(center, Vec2::new(image_width, image_height));
        let response = ui.put(
            image_rect,
            egui::Image::new(&self.logo).fit_to_exact_size(image_rect.size()),
        );
        response.context_menu(|ui| {
            ui.label(RichText::new("Doomsday Radio").color(GLOW));
            ui.collapsing("Schriftlizenz", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        ui.label(include_str!("../assets/ShareTechMono-OFL.txt"));
                    });
            });
        });
    }
}

impl eframe::App for RadioApp {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        let elapsed = self.last_frame.elapsed().as_secs_f32().min(0.1);
        self.last_frame = Instant::now();
        if context.input(|input| input.key_pressed(egui::Key::Space))
            && !context.wants_keyboard_input()
        {
            self.toggle();
        }
        let status = self
            .player
            .session
            .as_ref()
            .map(|session| session.status.lock().unwrap().clone());
        let mut samples = None;
        if let Some(session) = &self.player.session {
            if let Ok(snapshot) = session.snapshot.try_lock() {
                if snapshot.updated.elapsed() < Duration::from_millis(250) {
                    samples = Some((snapshot.samples, snapshot.sample_rate));
                }
            }
        }
        if let Some((samples, rate)) = samples {
            self.spectrum
                .update(&samples, rate, if self.muted { 0.0 } else { 1.0 }, elapsed);
        } else {
            self.spectrum.update(&[], 44_100, 0.0, elapsed);
        }
        let (status_text, status_color) = match &status {
            None => ("SIGNAL BEREIT", MUTED),
            Some(Status::Connecting) => ("VERBINDET", AMBER),
            Some(Status::Live) => ("LIVE", SIGNAL),
            Some(Status::Buffering) => ("SIGNAL PUFFERT", AMBER),
            Some(Status::Retry(_)) => ("NEUER VERSUCH", Color32::from_rgb(255, 90, 61)),
        };
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(18.0))
            .show(context, |ui| {
                let background = ui.max_rect();
                let mut vertical = background.top();
                while vertical < background.bottom() {
                    ui.painter().line_segment(
                        [
                            Pos2::new(background.left(), vertical),
                            Pos2::new(background.right(), vertical),
                        ],
                        Stroke::new(1.0_f32, Color32::from_black_alpha(14)),
                    );
                    vertical += 4.0;
                }
                self.station_header(ui);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if self.player.session.is_some() {
                            "NOW PLAYING"
                        } else {
                            "STREAM //"
                        })
                        .font(FontId::monospace(11.0))
                        .color(AMBER),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let response = ui.label(
                            RichText::new(status_text)
                                .monospace()
                                .size(11.0)
                                .color(status_color),
                        );
                        if let Some(Status::Retry(message)) = &status {
                            response.on_hover_text(message);
                        }
                    });
                });
                ui.add_space(10.0);
                self.visualizer(ui, (ui.available_height() - 116.0).max(28.0));
                ui.horizontal(|ui| {
                    ui.label(RichText::new("40 Hz").monospace().size(9.0).color(MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("16 kHz").monospace().size(9.0).color(MUTED));
                    });
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let playing = self.player.session.is_some();
                    let icon = if playing {
                        regular::STOP
                    } else {
                        regular::PLAY
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(icon).size(23.0).color(GLOW))
                                .fill(SURFACE)
                                .stroke(Stroke::new(1.0_f32, AMBER.gamma_multiply(0.65)))
                                .corner_radius(22.0)
                                .min_size(Vec2::new(44.0, 44.0)),
                        )
                        .on_hover_text(if playing { "Stoppen" } else { "Abspielen" })
                        .clicked()
                    {
                        self.toggle();
                    }
                    let icon = if self.muted || self.volume == 0.0 {
                        regular::SPEAKER_SLASH
                    } else {
                        regular::SPEAKER_HIGH
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(icon).size(21.0))
                                .frame(false)
                                .min_size(Vec2::new(28.0, 36.0)),
                        )
                        .on_hover_text(if self.muted {
                            "Ton einschalten"
                        } else {
                            "Stummschalten"
                        })
                        .clicked()
                    {
                        self.muted = !self.muted;
                    }
                    let slider_width = (ui.available_width() - 47.0).max(80.0);
                    ui.spacing_mut().slider_width = slider_width;
                    let changed = ui
                        .add(egui::Slider::new(&mut self.volume, 0.0..=1.0).show_value(false))
                        .on_hover_text("Lautstaerke")
                        .changed();
                    if changed {
                        self.muted = false;
                    }
                    ui.label(
                        RichText::new(format!("{:02}%", (self.volume * 100.0).round() as u32))
                            .monospace()
                            .size(11.0)
                            .color(MUTED),
                    );
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("DOOMSDAY.RADIO")
                            .monospace()
                            .size(10.0)
                            .color(MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let seconds = self.started.map_or(0, |start| start.elapsed().as_secs());
                        ui.label(
                            RichText::new(format!(
                                "{:02}:{:02}:{:02}",
                                seconds / 3600,
                                seconds / 60 % 60,
                                seconds % 60
                            ))
                            .monospace()
                            .size(10.0)
                            .color(MUTED),
                        );
                    });
                });
            });
        if let Some(session) = &self.player.session {
            session.set_volume(if self.muted { 0.0 } else { self.volume });
        }
        let active = self.player.session.is_some()
            || self.spectrum.levels.iter().any(|level| *level > 0.001);
        if active {
            context.request_repaint_after(Duration::from_millis(33));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_station_logo_is_small_and_nonblank() {
        let logo = image::load_from_memory(LOGO).unwrap().into_rgba8();
        assert_eq!(logo.dimensions(), (384, 391));
        assert!(LOGO.len() < 300_000);
        assert!(logo.pixels().any(|pixel| pixel[3] > 0 && pixel[0] > 200));
    }

    #[test]
    fn embedded_radio_font_has_a_truetype_header() {
        assert_eq!(&RADIO_FONT[..4], &[0, 1, 0, 0]);
        assert!(include_str!("../assets/ShareTechMono-OFL.txt").contains("SIL OPEN FONT LICENSE"));
    }
}
