#![deny(clippy::all)]
#![forbid(unsafe_code)]

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

mod frame_provider;
use frame_provider::FrameProvider;

use image::{ImageBuffer, RgbImage, RgbaImage};
use pixels::{Pixels, SurfaceTexture};
use std::collections::HashMap;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

const DEFAULT_WIDTH: u32 = 320;
const DEFAULT_HEIGHT: u32 = 240;


struct ApplicationState {
	cached_video_frames: HashMap<u64, RgbImage>,
	video_frames: FrameProvider
}

struct Display {
	window: Arc<Window>,
	pixels: Pixels,
}

fn _main(event_loop: EventLoop<()>) {
	let mut display: Option<Display> = None;

	let mut app_state = ApplicationState::new();

	let res = event_loop.run(|event, elwt| {
		elwt.set_control_flow(ControlFlow::Wait);
		match event {
			Event::Resumed => {
				let raw_window = elwt.create_window(Default::default()).unwrap();
				let window = Arc::new(raw_window);
				let pixels = {
					let window_size = window.inner_size();
					let surface_texture = SurfaceTexture::new(
						window_size.width,
						window_size.height,
						&window,
					);
					Pixels::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, surface_texture).unwrap()
				};
				window.request_redraw();
				display = Some(Display { window, pixels });
			}
			Event::Suspended => {
				display = None;
			}
			Event::WindowEvent {
				event: WindowEvent::RedrawRequested,
				..
			} => {
				if let Some(display) = &mut display {
					app_state.draw(display.pixels.frame_mut());
					display.pixels.render().unwrap();
					display.window.request_redraw();
				}
			}
			_ => {}
		}
		if display.is_some() {
			app_state.update();
		}
	});
	res.unwrap();
}

impl ApplicationState {
	/// Create a new `World` instance that can draw a moving box.
	fn new() -> Self {
		Self {
			cached_video_frames: Default::default(),
			video_frames: FrameProvider::NullFrameProvider,
		}
	}

	fn update(&mut self) {

	}

	/// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
	fn draw(&self, frame: &mut [u8]) {
		// Clear previous frame.
		frame.fill(0);

		for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
			let x = (i % DEFAULT_WIDTH as usize) as i16;
			let y = (i / DEFAULT_WIDTH as usize) as i16;

			let inside_the_box = false;

			let rgba = if inside_the_box {
				[0x5e, 0x48, 0xe8, 0xff]
			} else {
				[0x48, 0xb2, 0xe8, 0xff]
			};

			pixel.copy_from_slice(&rgba);
		}
	}
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
	use winit::platform::android::EventLoopBuilderExtAndroid;
	android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Info));
	let event_loop = EventLoopBuilder::new().with_android_app(app).build();
	log::info!("Hello from android!");
	_main(event_loop);
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info) // Default Log Level
		.parse_default_env()
		.init();
	let event_loop = EventLoop::new().unwrap();
	log::info!("Hello from desktop!");
	_main(event_loop);
}