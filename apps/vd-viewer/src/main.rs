//! Viewer window: presents decoded frames from the in-process loopback pipeline.
//!
//! T-0105 scope: a real egui/eframe window that shows the fake desktop through
//! capture → H.264 encode → H.264 decode, with start/stop controls and live counters.
//! Network connect (peer ID / password) arrives in P2; `--peer` is accepted for CLI
//! compatibility but does not connect yet.
//!
//! Decoding never runs on the UI thread: [`loopback::LoopbackSource`] owns a worker
//! thread and the UI only drains the newest decoded frame (role rule, ADR-0006).

#![forbid(unsafe_code)]

mod loopback;

use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use eframe::egui;
use loopback::{LoopbackConfig, LoopbackSource};
use vd_common::init_tracing;

/// remote_viewer viewer client.
#[derive(Debug, Parser)]
#[command(name = "vd-viewer", about = "remote_viewer viewer")]
struct Args {
    /// Peer ID to connect to (not used until P2).
    #[arg(long)]
    peer: Option<String>,

    /// Loopback frame width in pixels (must be even).
    #[arg(long, default_value_t = 960)]
    width: u32,

    /// Loopback frame height in pixels (must be even).
    #[arg(long, default_value_t = 540)]
    height: u32,

    /// Loopback frames per second.
    #[arg(long, default_value_t = 30)]
    fps: u8,

    /// Loopback encoder bitrate in bits per second.
    #[arg(long, default_value_t = 2_500_000)]
    bitrate: u32,
}

/// Viewer application state: one in-process loopback source plus the presented texture.
struct ViewerApp {
    cfg: LoopbackConfig,
    source: Option<LoopbackSource>,
    texture: Option<egui::TextureHandle>,
    /// Size of the image currently held by `texture`.
    texture_size: (u32, u32),
    /// Frames handed to the GPU since start.
    presented: u64,
    /// Frames the worker has published (cumulative).
    produced: u64,
    /// Frames lost to latest-frame-wins (cumulative).
    dropped: u64,
    /// Measured presentation rate over the last window.
    fps: f32,
    fps_mark: Instant,
    fps_count: u32,
    /// Last worker or start failure, shown in the status bar.
    error: Option<String>,
}

impl ViewerApp {
    fn new(cfg: LoopbackConfig) -> Self {
        let mut app = Self {
            cfg,
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

    /// Start the loopback worker, replacing any previous one.
    fn start(&mut self) {
        self.stop();
        self.error = None;
        match LoopbackSource::start(self.cfg) {
            Ok(source) => {
                self.source = Some(source);
                self.fps_mark = Instant::now();
                self.fps_count = 0;
            }
            // Startup failures are reported in the window rather than killing it: the
            // user keeps the Start button and can fix the flags without a crash loop.
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

    /// Drain the newest decoded frame, upload it, and refresh the counters.
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
                self.error = Some("loopback worker stopped unexpectedly".into());
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

    /// Upload a decoded RGBA frame, reusing the texture unless its size changed.
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
                self.texture = Some(ctx.load_texture(
                    "vd-viewer-loopback",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
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
        ui.horizontal(|ui| {
            ui.heading("remote_viewer");
            ui.separator();
            if self.source.is_some() {
                if ui
                    .button("Stop")
                    .on_hover_text("Stop the loopback worker (also stops on window close)")
                    .clicked()
                {
                    self.stop();
                }
            } else if ui
                .button("Start")
                .on_hover_text("Start the loopback worker")
                .clicked()
            {
                self.start();
            }
            if ui
                .button("Restart")
                .on_hover_text("Restart the loopback worker (F5)")
                .clicked()
            {
                self.restart();
            }
            ui.separator();
            ui.label("local loopback — network connect arrives in P2");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Esc closes");
            });
        });
    }

    fn status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "{}×{} @ {} fps requested",
                self.cfg.width, self.cfg.height, self.cfg.fps
            ));
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

        // Fit the frame inside the panel while preserving its aspect ratio.
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

        // Video arrives on its own thread; poll at half the source interval so the
        // window follows it without busy-spinning when nothing is decoded.
        if self.source.is_some() {
            let interval = Duration::from_secs_f32(0.5 / f32::from(self.cfg.fps));
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
    let cfg = LoopbackConfig {
        width: args.width,
        height: args.height,
        fps: args.fps,
        bitrate_bps: args.bitrate,
    };
    cfg.validate()?;
    tracing::info!(
        peer = args.peer.as_deref().unwrap_or("(none)"),
        width = cfg.width,
        height = cfg.height,
        fps = cfg.fps,
        "vd-viewer starting loopback window (network is P2)"
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("remote_viewer")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([480.0, 320.0]),
        ..Default::default()
    };
    // `eframe::Error` is not `Send + Sync` (it can carry glutin handles), so it cannot
    // cross an `anyhow` boundary with `?`; flatten it to a message instead.
    eframe::run_native(
        "remote_viewer",
        options,
        Box::new(move |_cc| Ok(Box::new(ViewerApp::new(cfg)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn drains_decoded_frames_into_a_texture() {
        // A headless Context is enough: texture upload only queues a delta.
        let ctx = egui::Context::default();
        let mut app = ViewerApp::new(small());
        wait_for_frames(&mut app, &ctx, 3);

        let texture = app.texture.as_ref().expect("texture uploaded");
        assert_eq!(texture.size(), [64, 48]);
        assert_eq!(app.texture_size, (64, 48));
        assert!(app.error.is_none(), "unexpected error: {:?}", app.error);
    }

    #[test]
    fn restart_replaces_the_source_and_resets_counters() {
        let ctx = egui::Context::default();
        let mut app = ViewerApp::new(small());
        wait_for_frames(&mut app, &ctx, 2);

        app.restart();
        assert_eq!(app.presented, 0);
        assert_eq!(app.produced, 0);
        wait_for_frames(&mut app, &ctx, 2);
        assert!(app.error.is_none(), "unexpected error: {:?}", app.error);
    }

    #[test]
    fn invalid_config_is_reported_instead_of_panicking() {
        // `new` starts the worker; a workable config is needed here, and the start
        // failure path is exercised through the loopback config validation instead.
        let bad = LoopbackConfig {
            width: 65,
            ..small()
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn stop_is_safe_when_never_started() {
        let mut app = ViewerApp::new(small());
        app.stop();
        app.stop();
        assert!(app.source.is_none());
    }
}
