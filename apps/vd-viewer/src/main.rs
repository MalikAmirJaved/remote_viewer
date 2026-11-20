//! Viewer window: loopback (default) or direct LAN connect (T-0208).
//!
//! Decoding never runs on the UI thread: a worker publishes decoded RGBA frames and
//! the UI only drains the newest one (role rule, ADR-0006).

#![forbid(unsafe_code)]

mod lan;
mod loopback;
mod source;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use eframe::egui;
use lan::LanClientConfig;
use loopback::LoopbackConfig;
use source::VideoSource;
use vd_common::init_tracing;

/// remote_viewer viewer client.
#[derive(Debug, Parser)]
#[command(name = "vd-viewer", about = "remote_viewer viewer")]
struct Args {
    /// Peer ID to connect to (signal path — unused until P3).
    #[arg(long)]
    peer: Option<String>,

    /// Connect directly to a LAN host (`vd-agent --listen`). Skips loopback.
    #[arg(long)]
    connect: Option<SocketAddr>,

    /// Pinned host certificate (DER) from `vd-agent --cert-out`.
    #[arg(long, default_value = "vd-agent-cert.der")]
    cert: PathBuf,

    /// TLS server name on the host certificate (SAN).
    #[arg(long, default_value = "localhost")]
    server_name: String,

    /// Loopback frame width in pixels (must be even). Ignored with `--connect`.
    #[arg(long, default_value_t = 960)]
    width: u32,

    /// Loopback frame height in pixels (must be even). Ignored with `--connect`.
    #[arg(long, default_value_t = 540)]
    height: u32,

    /// Loopback frames per second. Ignored with `--connect`.
    #[arg(long, default_value_t = 30)]
    fps: u8,

    /// Loopback encoder bitrate in bits per second. Ignored with `--connect`.
    #[arg(long, default_value_t = 2_500_000)]
    bitrate: u32,
}

#[derive(Debug, Clone)]
enum Mode {
    Loopback(LoopbackConfig),
    Lan(LanClientConfig),
}

/// Viewer application state.
struct ViewerApp {
    mode: Mode,
    /// Requested fps used only for UI repaint pacing.
    ui_fps: u8,
    source: Option<VideoSource>,
    texture: Option<egui::TextureHandle>,
    texture_size: (u32, u32),
    presented: u64,
    produced: u64,
    dropped: u64,
    fps: f32,
    fps_mark: Instant,
    fps_count: u32,
    error: Option<String>,
}

impl ViewerApp {
    fn new(mode: Mode, ui_fps: u8) -> Self {
        let mut app = Self {
            mode,
            ui_fps,
            source: None,
            texture: None,
            texture_size: (0, 0),
            presented: 0,
            produced: 0,
            dropped: 0,
            fps: 0.0,
            fps_mark: Instant::now(),
            fps_count: 0,
            error: None,
        };
        app.start();
        app
    }

    fn start(&mut self) {
        self.stop();
        self.error = None;
        let started = match &self.mode {
            Mode::Loopback(cfg) => VideoSource::start_loopback(*cfg),
            Mode::Lan(cfg) => VideoSource::start_lan(cfg.clone()),
        };
        match started {
            Ok(source) => {
                self.source = Some(source);
                self.fps_mark = Instant::now();
                self.fps_count = 0;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    fn stop(&mut self) {
        if let Some(mut source) = self.source.take() {
            source.stop();
        }
        self.fps = 0.0;
    }

    fn restart(&mut self) {
        self.presented = 0;
        self.produced = 0;
        self.dropped = 0;
        self.start();
    }

    fn drain(&mut self, ctx: &egui::Context) {
        let mut newest = None;
        if let Some(source) = &self.source {
            newest = source.try_recv_latest();
            self.produced = source.produced();
            self.dropped = source.dropped();
            if self.error.is_none() {
                self.error = source.last_error();
            }
            if self.error.is_none() && source.is_finished() {
                self.error = Some("video worker stopped unexpectedly".into());
            }
        }

        if let Some(frame) = newest {
            self.upload(ctx, &frame);
            self.presented = self.presented.saturating_add(1);
            self.fps_count = self.fps_count.saturating_add(1);
        }

        let elapsed = self.fps_mark.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.fps_count as f32 / elapsed.as_secs_f32();
            self.fps_count = 0;
            self.fps_mark = Instant::now();
        }
    }

    fn upload(&mut self, ctx: &egui::Context, frame: &vd_codec::DecodedFrame) {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.data,
        );
        match &mut self.texture {
            Some(texture) if self.texture_size == (frame.width, frame.height) => {
                texture.set(image, egui::TextureOptions::LINEAR);
            }
            _ => {
                self.texture =
                    Some(ctx.load_texture("vd-viewer-video", image, egui::TextureOptions::LINEAR));
                self.texture_size = (frame.width, frame.height);
            }
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let (close, restart) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::F5),
            )
        });
        if close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if restart {
            self.restart();
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let mode_label = self
            .source
            .as_ref()
            .map_or("stopped", VideoSource::mode_label);
        ui.horizontal(|ui| {
            ui.heading("remote_viewer");
            ui.separator();
            if self.source.is_some() {
                if ui.button("Stop").clicked() {
                    self.stop();
                }
            } else if ui.button("Start").clicked() {
                self.start();
            }
            if ui.button("Restart").on_hover_text("F5").clicked() {
                self.restart();
            }
            ui.separator();
            ui.label(mode_label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Esc closes");
            });
        });
    }

    fn status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            match &self.mode {
                Mode::Loopback(cfg) => {
                    ui.label(format!(
                        "{}×{} @ {} fps requested",
                        cfg.width, cfg.height, cfg.fps
                    ));
                }
                Mode::Lan(cfg) => {
                    ui.label(format!("LAN {}", cfg.connect));
                }
            }
            ui.separator();
            ui.label(format!("presented {}", self.presented));
            ui.label(format!("pipeline {}", self.produced));
            ui.label(format!("dropped {}", self.dropped));
            ui.label(format!("ui {:.1} fps", self.fps));
            if let Some(error) = &self.error {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("error: {error}")).color(egui::Color32::LIGHT_RED),
                );
            }
        });
    }

    fn video(&mut self, ui: &mut egui::Ui) {
        let Some(texture) = &self.texture else {
            ui.centered_and_justified(|ui| {
                ui.label("waiting for the first decoded frame…");
            });
            return;
        };

        let available = ui.available_size();
        let image = texture.size_vec2();
        let scale = (available.x / image.x).min(available.y / image.y).max(0.0);
        let size = image * scale;
        ui.allocate_ui_with_layout(
            available,
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.add(egui::Image::from_texture(texture).fit_to_exact_size(size));
            },
        );
    }
}

impl eframe::App for ViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.drain(&ctx);
        self.handle_keys(&ctx);

        egui::Panel::top("controls").show(ui, |ui| self.controls(ui));
        egui::Panel::bottom("status").show(ui, |ui| self.status(ui));
        egui::CentralPanel::default().show(ui, |ui| self.video(ui));

        if self.source.is_some() {
            let interval = Duration::from_secs_f32(0.5 / f32::from(self.ui_fps.max(1)));
            ctx.request_repaint_after(interval);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop();
    }
}

fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();

    let (mode, ui_fps) = if let Some(connect) = args.connect {
        let lan = LanClientConfig {
            connect,
            cert: args.cert,
            server_name: args.server_name,
        };
        lan.validate()?;
        tracing::info!(%connect, cert = %lan.cert.display(), "vd-viewer LAN connect");
        (Mode::Lan(lan), 30)
    } else {
        let cfg = LoopbackConfig {
            width: args.width,
            height: args.height,
            fps: args.fps,
            bitrate_bps: args.bitrate,
        };
        cfg.validate()?;
        tracing::info!(
            peer = args.peer.as_deref().unwrap_or("(none)"),
            "vd-viewer loopback window (use --connect for LAN)"
        );
        (Mode::Loopback(cfg), args.fps)
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("remote_viewer")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([480.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "remote_viewer",
        options,
        Box::new(move |_cc| Ok(Box::new(ViewerApp::new(mode, ui_fps)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn small() -> LoopbackConfig {
        LoopbackConfig {
            width: 64,
            height: 48,
            fps: 30,
            bitrate_bps: 500_000,
        }
    }

    fn wait_for_frames(app: &mut ViewerApp, ctx: &egui::Context, wanted: u64) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && app.presented < wanted {
            app.drain(ctx);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            app.presented >= wanted,
            "expected {wanted} presented frames, got {} (error: {:?})",
            app.presented,
            app.error
        );
    }

    #[test]
    fn cli_declares_connect_and_cert() {
        let mut cmd = Args::command();
        let help = cmd.render_long_help().to_string();
        assert!(help.contains("--connect"));
        assert!(help.contains("--cert"));
    }

    #[test]
    fn drains_decoded_frames_into_a_texture() {
        let ctx = egui::Context::default();
        let mut app = ViewerApp::new(Mode::Loopback(small()), 30);
        wait_for_frames(&mut app, &ctx, 3);

        let texture = app.texture.as_ref().expect("texture uploaded");
        assert_eq!(texture.size(), [64, 48]);
        assert_eq!(app.texture_size, (64, 48));
        assert!(app.error.is_none(), "unexpected error: {:?}", app.error);
    }

    #[test]
    fn restart_replaces_the_source_and_resets_counters() {
        let ctx = egui::Context::default();
        let mut app = ViewerApp::new(Mode::Loopback(small()), 30);
        wait_for_frames(&mut app, &ctx, 2);

        app.restart();
        assert_eq!(app.presented, 0);
        assert_eq!(app.produced, 0);
        wait_for_frames(&mut app, &ctx, 2);
        assert!(app.error.is_none(), "unexpected error: {:?}", app.error);
    }

    #[test]
    fn invalid_loopback_config_is_rejected() {
        let bad = LoopbackConfig {
            width: 65,
            ..small()
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn stop_is_safe_when_never_started() {
        let mut app = ViewerApp::new(Mode::Loopback(small()), 30);
        app.stop();
        app.stop();
        assert!(app.source.is_none());
    }
}
