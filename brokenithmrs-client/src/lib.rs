mod client;
mod egui_tools;
mod fps_counter;
mod jni;
use crate::egui_tools::EguiRenderer;
use android_activity::WindowManagerFlags;
use egui::FontFamily::*;
use egui::{Color32, FontId, Pos2, TextStyle, Vec2};
use egui_wgpu::{wgpu, ScreenDescriptor};
use preferences::{AppInfo, Preferences};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::event::{self, KeyEvent, Touch, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::Key;
use winit::window::{Window, WindowId};
pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
}
const APP_INFO: AppInfo = AppInfo {
    name: "brokenithmrs",
    author: "stupichvsrg",
};
impl AppState {
    async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Rgba8Unorm;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("fuck!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
            desired_maximum_frame_latency: 0,
            alpha_mode: wgpu::CompositeAlphaMode::Inherit,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, window);

        let scale_factor = 1.0;
        Self {
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            scale_factor,
        }
    }
    fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
pub struct App {
    setting_open: bool,
    hold_length: f64,
    touch_info: HashMap<u64, Pos2>,
    air_section: [char; 6],
    slider_section: [char; 32],
    activated_slider_section_stored: HashMap<String, bool>,
    activated_slider_section_current: HashMap<bool, String>,
    last_deactivated_slider_section: String,
    air_start_pos: f32,
    activated_air_section_stored: HashMap<String, bool>,
    activated_air_section_current: HashMap<bool, String>,
    last_deactivated_air_section: String,
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
    touch_pos: Pos2,
    color_slider: Color32,
    color_air: Color32,
    fpscounter: fps_counter::FPSCounter,
    air_slider_separator: f32,
    ip: String,
    path: String,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Prefs {
    ip: String,
    airseppos: f32,
    airpos: f32,
}
impl App {
    pub fn new() -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let path = format!("/data/user/0/com.example.brokenithmrs/files");
        let mut ip = format!("");
        let mut air_slider_separator = 0.4;
        if let Ok(load_prefs) = Prefs::load(&APP_INFO, path.clone()) {
            ip = load_prefs.ip;
            air_slider_separator = load_prefs.airseppos;
        }
        let mut activated_slider_section_current = HashMap::new();
        activated_slider_section_current.insert(false, format!("0,0,0"));
        activated_slider_section_current.insert(true, format!("0,0,0"));
        let mut activated_slider_section_stored = HashMap::new();
        activated_slider_section_stored.insert("0,0,0".to_string(), true);

        let mut activated_air_section_current = HashMap::new();
        activated_air_section_current.insert(false, format!("0,0,0"));
        activated_air_section_current.insert(true, format!("0,0,0"));
        let mut activated_air_section_stored = HashMap::new();
        activated_air_section_stored.insert("0,0,0".to_string(), true);
        let slider_section: [char; 32] = [
            '5', '3', '1', 'Y', 'W', 'U', 'S', 'Q', 'O', 'M', 'K', 'I', 'G', 'E', 'C', 'A', '6',
            '4', '2', 'Z', 'X', 'V', 'T', 'R', 'P', 'N', 'L', 'J', 'H', 'F', 'D', 'B',
        ];
        let air_section: [char; 6] = [';', '\\', ']', '[', '=', '-'];
        Self {
            air_start_pos: 0.0,
            air_section,
            slider_section,
            last_deactivated_slider_section: format!(""),
            last_deactivated_air_section: format!(""),
            path,
            ip,
            air_slider_separator,
            activated_slider_section_stored,
            activated_slider_section_current,
            activated_air_section_stored,
            activated_air_section_current,
            fpscounter: fps_counter::FPSCounter::default(),
            setting_open: true,
            hold_length: 0.0,
            touch_info: HashMap::new(),
            touch_pos: Pos2 {
                x: 0 as f32,
                y: 0 as f32,
            },
            color_slider: Color32::TRANSPARENT,
            color_air: Color32::TRANSPARENT,
            instance,
            state: None,
            window: None,
        }
    }
    #[no_mangle]
    fn android_main(app: android_activity::AndroidApp) -> Result<(), Box<dyn Error>> {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        std::env::set_var("RUST_BACKTRACE", "full");
        let mut event_loop_builder = EventLoop::builder();

        app.set_window_flags(WindowManagerFlags::FULLSCREEN, WindowManagerFlags::empty());
        event_loop_builder.with_android_app(app.clone());
        let event_loop = event_loop_builder.build()?;
        let mut app_instance = App::new();
        event_loop.run_app(&mut app_instance).map_err(Into::into)
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1;
        let initial_height = 1;
        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));
        let surface = self.instance.create_surface(window.clone());

        let state = AppState::new(
            &self.instance,
            surface.unwrap(),
            &window,
            initial_width,
            initial_height,
        )
        .await;
        let text: BTreeMap<TextStyle, FontId> = [
            (TextStyle::Heading, FontId::new(50.0, Proportional)),
            (TextStyle::Body, FontId::new(50.0, Proportional)),
            (TextStyle::Monospace, FontId::new(50.0, Monospace)),
            (TextStyle::Button, FontId::new(50.0, Proportional)),
            (TextStyle::Small, FontId::new(50.0, Proportional)),
        ]
        .into();
        window.set_ime_allowed(true);

        state
            .egui_renderer
            .context()
            .all_styles_mut(move |style| style.text_styles = text.clone());
        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        self.state.as_mut().unwrap().resize_surface(width, height);
    }
    fn handle_redraw(&mut self) {
        let fps = self.fpscounter.tick();
        let delta: f64 = 60.0 / fps as f64;

        let state = self.state.as_mut().unwrap();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: 1.0 as f32,
        };
        let surface_texture = state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let window = self.window.as_ref().unwrap();
        {
            state.egui_renderer.begin_frame(window);
            let screen_size = state.egui_renderer.context().screen_rect();
            if (self.hold_length > 1200.0 / delta) || self.setting_open {
                self.setting_open = true;
                egui::Window::new("Settings")
                    .default_open(true)
                    .collapsible(false)
                    .resizable(true)
                    .movable(false)
                    .fixed_pos(state.egui_renderer.context().screen_rect().center())
                    .auto_sized()
                    .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
                    .show(state.egui_renderer.context(), |ui| {
                        ui.vertical(|ui| {
                            let air_slider_sep = self.air_slider_separator.clone();
                            let air_pos = self.air_start_pos.clone();
                            let mut fuck = self.ip.clone();

                            ui.style_mut().interaction.selectable_labels = false;
                            ui.label("air position");
                            ui.style_mut().spacing.slider_width = 425.0;
                            ui.add(
                                egui::Slider::new(&mut self.air_start_pos, 0.0..=0.1)
                                    .show_value(false)
                                    .text(format!(" {}", air_pos)),
                            );

                            ui.label("air/slider separator position");
                            ui.style_mut().spacing.slider_width = 425.0;
                            ui.add(
                                egui::Slider::new(&mut self.air_slider_separator, 0.4..=0.6)
                                    .show_value(false)
                                    .text(format!(" {}", air_slider_sep)),
                            );
                            ui.label("ip address to send inputs to");
                            let mut text = egui::TextEdit::singleline(&mut fuck)
                                .desired_width(425.0)
                                .show(ui);
                            text.response.clicked().then(|| jni::show_soft_input(true));

                            self.ip = fuck;
                            ui.label(format!("fps: {}", fps));
                        })
                    });
            }

            egui::CentralPanel::default().show(state.egui_renderer.context(), |ui| {
                ui.vertical(|ui| {
                    if (ui.input(|i| i.any_touches())) {
                        self.hold_length += 1.0;
                    } else {
                        self.hold_length = 0.0;
                    }
                    ui.add_space(screen_size.height() * self.air_start_pos);

                    ui.spacing_mut().item_spacing = [0.0; 2].into();
                    ui.horizontal(|ui| {
                        ui.set_max_height(
                            (screen_size.height() * self.air_slider_separator)
                                - (screen_size.height() * self.air_start_pos),
                        );
                        let _ = egui_extras::StripBuilder::new(ui)
                            .sizes(egui_extras::Size::relative((6 as f32).recip()), 6)
                            .vertical(|mut strip| {
                                for r in self.air_section.into_iter() {
                                    strip.cell(|ui| {
                                        let response = ui.allocate_response(
                                            [ui.max_rect().width(), ui.max_rect().height()].into(),
                                            egui::Sense::drag(),
                                        );
                                        let mut held = false;
                                        let mut activated_section = format!("{}", r);

                                        let mut deactivated_section = format!("{}", r);

                                        for (id, pos) in self.touch_info.clone() {
                                            if (response.interact_rect.contains(pos)) {
                                                if self
                                                    .activated_air_section_stored
                                                    .get(&format!("{},{}", id, r))
                                                    != Some(&true)
                                                {
                                                    activated_section = format!("{},{}", id, r);
                                                    held = true;
                                                    self.activated_air_section_current
                                                        .insert(held, activated_section.clone());
                                                    client::send_keys(self.ip.clone(), r, true);
                                                    self.activated_air_section_stored
                                                        .insert(activated_section, true);
                                                }
                                                self.color_air = Color32::LIGHT_BLUE;
                                            } else if !response.interact_rect.contains(pos) {
                                                if !self.activated_air_section_stored.is_empty() {
                                                    if (self
                                                        .activated_air_section_stored
                                                        .get(&format!("{},{}", id, r)))
                                                        == Some(&true)
                                                    {
                                                        deactivated_section =
                                                            format!("{},{}", id, r);
                                                        held = false;

                                                        self.activated_air_section_current.insert(
                                                            held,
                                                            deactivated_section.clone(),
                                                        );
                                                        client::send_keys(
                                                            self.ip.clone(),
                                                            r,
                                                            false,
                                                        );
                                                        self.activated_air_section_stored
                                                            .remove(&deactivated_section);
                                                    }
                                                }
                                                self.color_air = Color32::TRANSPARENT;
                                            }

                                            ui.painter().rect(
                                                response.rect,
                                                egui::Rounding {
                                                    sw: 0.0,
                                                    se: 0.0,
                                                    ne: 0.0,
                                                    nw: 0.0,
                                                },
                                                self.color_air,
                                                egui::Stroke {
                                                    width: 1.0,
                                                    color: Color32::LIGHT_BLUE,
                                                },
                                            );
                                        }
                                    });
                                }
                            });
                    });
                    let _ = egui_extras::StripBuilder::new(ui)
                        .sense(egui::Sense::click_and_drag())
                        .sizes(egui_extras::Size::relative((2 as f32).recip()), 2)
                        .vertical(|mut strip| {
                            for i in 0..2 {
                                let slider: &[char];
                                if (i == 0) {
                                    slider = &self.slider_section[0..16];
                                } else {
                                    slider = &self.slider_section[16..32];
                                }
                                strip.cell(|ui| {
                                    egui_extras::StripBuilder::new(ui)
                                        .sizes(egui_extras::Size::relative((16 as f32).recip()), 16)
                                        .horizontal(|mut strip| {
                                            for s in slider.into_iter() {
                                                strip.cell(|ui| {
                                            let response = ui.allocate_response(
                                                [
                                                    ui.max_rect().width(),
                                                    ui.max_rect().height(),
                                                ]
                                                .into(),
                                                egui::Sense::drag(),
                                            );
                                            let mut held = false;
                                            let mut activated_section = format!("{}", s);

                                            let mut deactivated_section = format!("{}", s);
                                            if response.drag_started
                                                && (self.setting_open)
                                                && (*s == '1'
                                                    || *s == '2'
                                                    || *s == 'y'
                                                    || *s == 'z')
                                            {
                                                self.setting_open = false;
                                            }
                                            for (id, pos) in self.touch_info.clone() {
                                                if (response.interact_rect.contains(pos)) {
                                                    if self
                                                        .activated_slider_section_stored
                                                        .get(&format!("{},{}", id, s))
                                                        != Some(&true)
                                                    {
                                                        activated_section = format!("{},{}", id, s);
                                                        held = true;
                                                        self.activated_slider_section_current
                                                            .insert(
                                                                held,
                                                                activated_section.clone(),
                                                            );

                                                            client::send_keys(
                                                             self.ip.clone(),
                                                         *s,
                                                            true,
                                                        );
                                                        self.activated_slider_section_stored
                                                            .insert(activated_section, true);
                                                    }
                                                    self.color_slider = Color32::ORANGE;
                                                } else if !response.interact_rect.contains(pos) {
                                                    if !self
                                                        .activated_slider_section_stored
                                                        .is_empty()
                                                    {
                                                        if (self
                                                            .activated_slider_section_stored
                                                            .get(&format!("{},{}", id, s)))
                                                            == Some(&true)
                                                        {
                                                            deactivated_section =
                                                                format!("{},{}", id, s);
                                                            held = false;
                                                            self.activated_slider_section_current
                                                                .insert(
                                                                    held,
                                                                    deactivated_section.clone(),
                                                                );

                                                            client::send_keys(
                                                            self.ip.clone(),
                                                                        *s,
                                                            false,
                                                        );
                                                            self.activated_slider_section_stored
                                                                .remove(&deactivated_section);
                                                        }
                                                    }
                                                    self.color_slider = Color32::TRANSPARENT;
                                                }
                                                if (self.setting_open)
                                                    && (*s == '1'
                                                        || *s == '2'
                                                        || *s == 'y'
                                                        || *s == 'z')
                                                {
                                                    self.color_slider = Color32::DARK_RED;
                                                }
                                                ui.painter().rect(
                                                    response.rect,
                                                    egui::Rounding {
                                                        sw: 0.0,
                                                        se: 0.0,
                                                        ne: 0.0,
                                                        nw: 0.0,
                                                    },
                                                    self.color_slider,
                                                    egui::Stroke {
                                                        width: 1.0,
                                                        color: Color32::ORANGE,
                                                    },
                                                );
                                            }
                                    });
                                            }
                                        });
                                });
                            }
                        });
                })
            });

            state.egui_renderer.end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        jni::hide_ui();
        pollster::block_on(self.set_window(window))
    }
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let mut temp_prefs = Prefs {
            ip: self.ip.clone(),
            airseppos: self.air_slider_separator.clone(),
            airpos: self.air_start_pos.clone(),
        };
        temp_prefs.save(&APP_INFO, self.path.clone());
        std::process::exit(69);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // let egui render to process the event first
        self.state
            .as_mut()
            .unwrap()
            .egui_renderer
            .handle_input(&mut self.window.as_ref().unwrap(), &event);

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key.as_ref() {
                Key::Character("1") => self.ip += "1",

                Key::Character("2") => self.ip += "2",
                Key::Character("3") => self.ip += "3",
                Key::Character("4") => self.ip += "4",
                Key::Character("5") => self.ip += "5",
                Key::Character("6") => self.ip += "6",
                Key::Character("7") => self.ip += "7",
                Key::Character("8") => self.ip += "8",
                Key::Character("9") => self.ip += "9",
                Key::Character("0") => self.ip += "0",
                Key::Character(".") => self.ip += ".",
                _ => {}
            },
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            WindowEvent::Touch(Touch {
                phase,
                location,
                id,
                ..
            }) => match phase {
                event::TouchPhase::Started => {
                    self.touch_pos.x = location.x as f32;
                    self.touch_pos.y = location.y as f32;
                    self.touch_info.insert(id, self.touch_pos);
                }
                event::TouchPhase::Moved => {
                    self.touch_pos.x = location.x as f32;
                    self.touch_pos.y = location.y as f32;
                    self.touch_info.insert(id, self.touch_pos);
                }
                event::TouchPhase::Cancelled | event::TouchPhase::Ended => {
                    self.touch_pos.x = 0 as f32;
                    self.touch_pos.y = 0 as f32;
                    self.touch_info.insert(id, self.touch_pos);
                }
            },
            _ => (),
        }
    }
}
