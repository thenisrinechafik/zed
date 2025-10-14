#![cfg(target_os = "windows")]

use std::time::{Duration, Instant};

use gpui::{
    App, Application, Bounds, Context, KeyDownEvent, MouseMoveEvent, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};

struct SmokeProbe {
    last_frame: Instant,
    last_sample: Instant,
    fps: f32,
    frames: u32,
}

impl SmokeProbe {
    fn new(_window: &mut Window, _: &mut Context<Self>) -> Self {
        log_configuration();
        SmokeProbe {
            last_frame: Instant::now(),
            last_sample: Instant::now(),
            fps: 0.0,
            frames: 0,
        }
    }

    fn on_key(&mut self, event: &KeyDownEvent, _: &mut Window, _: &mut Context<Self>) {
        log::info!(
            target: "win_smoke",
            "key={:?} modifiers={:?}",
            event.key,
            event.modifiers
        );
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, _: &mut Context<Self>) {
        log::trace!(
            target: "win_smoke",
            "mouse=({}, {})",
            event.position.x,
            event.position.y
        );
    }
}

impl Render for SmokeProbe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        let frame_delta = now - self.last_frame;
        self.last_frame = now;
        self.frames += 1;
        if now - self.last_sample >= Duration::from_millis(500) {
            self.fps = self.frames as f32 * 1_000.0
                / (now - self.last_sample).as_millis().max(1) as f32;
            self.frames = 0;
            self.last_sample = now;
            log::debug!(target: "win_smoke", "frame_time={:?} fps={:.2}", frame_delta, self.fps);
        }

        window.request_animation_frame();

        let status = format!("{:.1} fps", self.fps);
        div()
            .flex()
            .size_full()
            .bg(rgb(0x202830))
            .justify_center()
            .items_center()
            .text_color(gpui::white())
            .text_xl()
            .on_key_down(cx.listener(SmokeProbe::on_key))
            .on_mouse_move(cx.listener(SmokeProbe::on_mouse_move))
            .child(
                div()
                    .padding(px(16.0))
                    .bg(rgb(0x2d3845))
                    .rounded_lg()
                    .shadow_lg()
                    .child("Hello Frame!")
                    .child(div().text_sm().text_color(gpui::gray(0.6)).child(status)),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        cx.open_window(options, |window, cx| cx.new(|cx| SmokeProbe::new(window, cx)))
            .expect("failed to open smoke window");
        cx.activate(true);
    });
}

fn log_configuration() {
    let vsync = std::env::var("ZED_VSYNC").unwrap_or_else(|_| "auto".into());
    let hdr = std::env::var("ZED_HDR").unwrap_or_else(|_| "auto".into());
    let composition = std::env::var("ZED_DX_COMPOSITION").unwrap_or_else(|_| "auto".into());
    log::info!(
        target: "win_smoke",
        "smoke launch (vsync={vsync}, hdr={hdr}, composition={composition})"
    );
}
