//! The browser side: WebGPU device and canvas surface, the kit, one cluster, input and the
//! animation loop.
use crate::sim::Sim;
use fh6_core::{AppConfig, ClusterTheme};
use fh6_kit::wgpu;
use fh6_kit::{vec2, Cluster, FrameInput, Kit, Layout, Logo, Palette, SignalSmoother, Tokens, View};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// The design tokens, embedded so the page needs no extra fetch.
const TOKENS: &str = include_str!("../../../data/design_tokens.json");

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

fn status(document: &web_sys::Document, msg: &str) {
    if let Some(el) = document.get_element_by_id("status") {
        el.set_text_content(Some(msg));
    }
}

struct App {
    canvas: web_sys::HtmlCanvasElement,
    dpr: f64,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    kit: Kit,
    cluster: Box<dyn Cluster>,
    palette: Palette,
    tokens: Tokens,
    app_config: AppConfig,
    logo: Logo,
    smoother: SignalSmoother,
    sim: Sim,
    throttle: bool,
    last_ms: f64,
    t: f32,
}

impl App {
    fn frame(&mut self, now_ms: f64) {
        let dt = if self.last_ms > 0.0 {
            ((now_ms - self.last_ms) / 1000.0) as f32
        } else {
            1.0 / 60.0
        };
        self.last_ms = now_ms;
        self.t += dt;
        // Follow the canvas' CSS size (the page may lay out after start-up, or be resized).
        let w = ((f64::from(self.canvas.client_width()) * self.dpr) as u32).max(1);
        let h = ((f64::from(self.canvas.client_height()) * self.dpr) as u32).max(1);
        if (w, h) != (self.config.width, self.config.height) {
            self.canvas.set_width(w);
            self.canvas.set_height(h);
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.device, &self.config);
        }
        self.sim.step(dt, self.throttle);
        let data = self.sim.telemetry();
        let signals = self.smoother.update(&data, &self.app_config, &self.tokens, dt);
        let fonts = self.kit.font_choice(None);
        self.cluster.update(&FrameInput {
            telemetry: &data,
            signals: &signals,
            dt,
            t: self.t,
            config: &self.app_config,
            palette: &self.palette,
            tokens: &self.tokens,
            fonts,
            logo: &self.logo,
            view: View::Cockpit,
            reduced_flash: false,
        });
        let (w, h) = (self.config.width, self.config.height);
        // Fit the cockpit envelope (1440 x 560 local px) into 88% of the canvas.
        let scale = (w as f32 * 0.88 / 1440.0).min(h as f32 * 0.88 / 560.0);
        let layout = Layout::centered(vec2(w as f32 * 0.5, h as f32 * 0.5 + 40.0 * scale), scale, (w, h));
        {
            let mut p = self.kit.begin_frame(&layout);
            self.cluster.draw(&mut p, &layout);
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("web-preview frame"),
        });
        self.kit.render(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            self.config.format,
            (w, h),
            1.0,
        );
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
    }
}

/// Entry point, run by the page's module loader.
#[wasm_bindgen(start)]
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = run().await {
            log(&format!("web-preview: {e}"));
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                status(&doc, &format!("Error: {e}"));
            }
        }
    });
}

async fn run() -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas: web_sys::HtmlCanvasElement = document
        .get_element_by_id("preview")
        .ok_or("no #preview canvas")?
        .dyn_into()
        .map_err(|_| "#preview is not a canvas")?;
    let dpr = window.device_pixel_ratio();
    let width = ((f64::from(canvas.client_width()) * dpr) as u32).max(1);
    let height = ((f64::from(canvas.client_height()) * dpr) as u32).max(1);
    canvas.set_width(width);
    canvas.set_height(height);

    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = wgpu::Backends::BROWSER_WEBGPU;
    let instance = wgpu::Instance::new(desc);
    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
        .map_err(|e| format!("WebGPU surface: {e}"))?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            ..Default::default()
        })
        .await
        .map_err(|e| format!("no WebGPU adapter ({e}); use a browser with WebGPU enabled"))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("web-preview device"),
            required_limits: wgpu::Limits::default().using_resolution(adapter.limits()),
            ..Default::default()
        })
        .await
        .map_err(|e| format!("WebGPU device: {e}"))?;
    let caps = surface.get_capabilities(&adapter);
    // The kit writes sRGB-encoded premultiplied values, so pick a non-sRGB format.
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| !f.is_srgb())
        .or_else(|| caps.formats.first().copied())
        .ok_or("the canvas offers no format")?;
    let alpha_mode = if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
        wgpu::CompositeAlphaMode::PreMultiplied
    } else {
        caps.alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto)
    };
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 1,
        alpha_mode,
        view_formats: Vec::new(),
    };
    surface.configure(&device, &config);

    status(&document, "Generating glyphs…");
    let tokens = Tokens::from_json(TOKENS).unwrap_or_default();
    let mut kit = Kit::new(&device, &queue, tokens.clone()).map_err(|e| e.to_string())?;
    let search = window.location().search().unwrap_or_default();
    // `?cluster=<Theme>` picks any catalog cluster (e.g. `?cluster=VernierLoupe`); the older
    // `?analog` still selects PorscheAnalog.
    let named = search
        .trim_start_matches('?')
        .split('&')
        .find_map(|kv| kv.strip_prefix("cluster="))
        .and_then(ClusterTheme::from_name);
    let theme = named.unwrap_or(if search.contains("PorscheAnalog") || search.contains("analog") {
        ClusterTheme::PorscheAnalog
    } else {
        ClusterTheme::LiquidGlass
    });
    let entry = fh6_clusters::registry::resolve(theme);
    let cluster = {
        let mut setup = kit.setup(&device, &queue, None);
        (entry.make)(&mut setup)
    };
    let palette = Palette::family(&tokens, entry.family.key());
    kit.set_palette(palette);
    status(
        &document,
        &format!("{} · hold Space for throttle, 1–9 for gears", entry.name),
    );

    let app = Rc::new(RefCell::new(App {
        canvas,
        dpr,
        device,
        queue,
        surface,
        config,
        kit,
        cluster,
        palette,
        tokens,
        app_config: AppConfig::default(),
        logo: Logo {
            custom_name: "FH6 WheelDash".into(),
            ..Logo::default()
        },
        smoother: SignalSmoother::new(),
        sim: Sim::new(),
        throttle: false,
        last_ms: 0.0,
        t: 0.0,
    }));

    // Keyboard: Space = throttle, 1–9 = gears.
    {
        let app = Rc::clone(&app);
        let down = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            let key = e.key();
            let Ok(mut a) = app.try_borrow_mut() else { return };
            if key == " " {
                a.throttle = true;
                e.prevent_default();
            } else if let Some(g) = key.chars().next().and_then(|c| c.to_digit(10)) {
                if (1..=9).contains(&g) {
                    a.sim.shift(g as u8);
                }
            }
        });
        window
            .add_event_listener_with_callback("keydown", down.as_ref().unchecked_ref())
            .map_err(|_| "keydown listener")?;
        down.forget();
    }
    {
        let app = Rc::clone(&app);
        let up = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            if e.key() == " " {
                if let Ok(mut a) = app.try_borrow_mut() {
                    a.throttle = false;
                }
            }
        });
        window
            .add_event_listener_with_callback("keyup", up.as_ref().unchecked_ref())
            .map_err(|_| "keyup listener")?;
        up.forget();
    }

    // requestAnimationFrame loop.
    let tick: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let tick_again = Rc::clone(&tick);
    *tick.borrow_mut() = Some(Closure::new(move |now: f64| {
        if let Ok(mut a) = app.try_borrow_mut() {
            a.frame(now);
        }
        if let (Some(w), Some(cb)) = (web_sys::window(), tick_again.borrow().as_ref()) {
            let _ = w.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    if let Some(cb) = tick.borrow().as_ref() {
        window
            .request_animation_frame(cb.as_ref().unchecked_ref())
            .map_err(|_| "requestAnimationFrame")?;
    }
    Ok(())
}
