#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::cell::Cell;
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
mod frame_provider;

use egui::{emath, vec2, Color32, ColorImage, Context, Frame, Image, Pos2, Rect, Sense, Stroke, Ui, Window, TextureFilter, TextureHandle, TextureOptions, pos2};
use image::{ImageBuffer, RgbImage, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;

use crate::frame_provider::{FrameProvider, get_frame_provider, image_to_egui_image, NullFrameProvider, ImageSequenceFrameProvider};
// Huge props to egui-video which I could not use but was instrumental to figuring this out.
// Look at https://github.com/n00kii/egui-video/blob/main/src/lib.rs for overlap.

//#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[cfg_attr(feature = "serde", serde(default))]
pub struct AutoRoto {
	composition_properties: CompositionProperties,

	points: HashMap<usize, Vec<Pos2>>,  // frame number to list of positions in 0-1 normalized form.
	lines: Vec<Vec<Pos2>>, 	// in 0-1 normalized coordinates
	stroke: Stroke,

	frame_provider: Box<dyn FrameProvider>,
	cached_frames: HashMap<usize, image::RgbaImage>,
	current_frame: usize,

	display_texture_options: TextureOptions,
	display_texture_handle: TextureHandle,
	display_texture_frame: usize, // The current image frame loaded into the texture.
}

pub struct CompositionProperties {
	width: u64,
	height: u64,
	framerate: u32, // TODO: Support NTSC 29.97FPS. Ugh.
}

impl Default for CompositionProperties {
	fn default() -> Self {
		Self {
			width: 0,
			height: 0,
			framerate: 60,
		}
	}
}

impl AutoRoto {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		egui_extras::install_image_loaders(&cc.egui_ctx);

		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			//return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Self {
			composition_properties: Default::default(),
			points: Default::default(),
			lines: Default::default(),
			stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
			frame_provider: Box::new(NullFrameProvider::new(512, 512)),
			cached_frames: Default::default(),
			current_frame: 0,
			display_texture_handle: cc.egui_ctx.load_texture("display_image", ColorImage::example(), TextureOptions::default()),
			display_texture_options: TextureOptions::default(),
			display_texture_frame: 0,
		}
	}

	fn set_frame_provider(&mut self, new_frame_provider: Box<dyn FrameProvider>) {
		self.frame_provider = new_frame_provider;
		self.current_frame = 0;
		self.cached_frames.clear();
		self.display_texture_frame += 1; // HACK: This forces a refresh of the GPU texture from the CPU texture.

		// TODO: Scan all frames instead of just the first.
		let f = &self.frame_provider.get_frame(0);
		self.composition_properties.width = f.width() as u64;
		self.composition_properties.height = f.height() as u64;
	}

	fn load_and_move_current_frame_to_gpu(&mut self) {
		if self.current_frame != self.display_texture_frame {
			if !self.cached_frames.contains_key(&self.current_frame) {
				// Load and cache the frame.
				let img = self.frame_provider.get_frame(self.current_frame as u64);
				self.cached_frames.insert(self.current_frame, img);
			}

			let cached_frame_ref = self.cached_frames.get(&self.current_frame).expect("CANNOT FETCH BACK FRAME JUST ADDED TO LIST.");

			self.display_texture_handle.set(
				image_to_egui_image(cached_frame_ref),
				self.display_texture_options
			);
			self.display_texture_frame = self.current_frame;
		}
	}

	/// Create the [`egui::Image`] for the video frame.
	fn generate_frame_image(&self, size: egui::Vec2) -> Image {
		Image::new(egui::load::SizedTexture::new(self.display_texture_handle.id(), size)).sense(Sense::click())
	}

	/// Draw the video frame with a specific rect (without controls).
	fn render_frame(&self, ui: &mut Ui, size: egui::Vec2) -> egui::Response {
		ui.add(self.generate_frame_image(size))
	}

	/// Draw the video frame (without controls).
	fn render_frame_at(&self, ui: &mut Ui, rect: Rect) -> egui::Response {
		ui.put(rect, self.generate_frame_image(rect.size()))
	}

	pub fn ui_timeline(&mut self, ui: &mut Ui) -> egui::Response {
		todo!()
	}

	pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
		ui.horizontal(|ui| {
			ui.label("Stroke:");
			ui.add(&mut self.stroke);
			ui.separator();
			if ui.button("Clear Painting").clicked() {
				self.lines.clear();
			}
		})
			.response
	}

	fn draw_video_frame_with_overlay(&mut self, ui: &mut Ui) -> egui::Response {
		let (mut response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

		let available_rect = Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions());
		let to_screen = emath::RectTransform::from_to(available_rect, response.rect,);
		let from_screen = to_screen.inverse();

		self.load_and_move_current_frame_to_gpu();
		// This matches the aspect ratio of the output pane, which isn't quite what we want:
		//painter.image(self.display_texture_handle.id(), response.rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
		//self.render_frame(ui, egui::Vec2::new(0.0, 0.0));
		painter.image(
			self.display_texture_handle.id(),
			Rect::from_min_max(
				pos2(0f32, 0f32),
				pos2(self.composition_properties.width as f32, self.composition_properties.height as f32)
			),
			Rect::from_min_max(
				Pos2::ZERO,
				Pos2::new(1.0, 1.0)
			),
			Color32::WHITE
		);

		if self.lines.is_empty() {
			self.lines.push(vec![]);
		}

		let current_line = self.lines.last_mut().unwrap();

		if let Some(pointer_pos) = response.interact_pointer_pos() {
			let canvas_pos = from_screen * pointer_pos;
			if current_line.last() != Some(&canvas_pos) {
				current_line.push(canvas_pos);
				response.mark_changed();
			}
		} else if !current_line.is_empty() {
			self.lines.push(vec![]);
			response.mark_changed();
		}

		let shapes = self
			.lines
			.iter()
			.filter(|line| line.len() >= 2)
			.map(|line| {
				let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
				egui::Shape::line(points, self.stroke)
			});

		painter.extend(shapes);

		response
	}

	fn draw_menu(&mut self, ctx: &Context) -> egui::Response {
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:
			egui::menu::bar(ui, |ui| {
				let is_web = cfg!(target_arch = "wasm32");
				ui.menu_button("File", |ui| {
					if ui.button("Open Image Sequence").clicked() {
						//if let Some(files) = rfd::FileDialog::new().pick_files() {}
						if let Some(fp) = get_frame_provider(true) {
							self.set_frame_provider(fp);
						}
						ui.close_menu();
					}
					//if ui.button("Open Video").clicked() {}
					if ui.button("Quit").clicked() {
						ctx.send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});
				ui.add_space(16.0);

				egui::widgets::global_theme_preference_buttons(ui);
			});
		}).response
	}

	fn update_and_draw(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		self.draw_menu(ctx);
		self.draw_controls(ctx);

		// Main panel:
		egui::CentralPanel::default().show(ctx, |ui| {
			//ui.label("Write something: ");
			//ui.text_edit_singleline(&mut self.label);

			Frame::canvas(ui.style()).show(ui, |ui| {
				self.draw_video_frame_with_overlay(ui);
			});

			ui.separator();
		});
	}

	pub fn draw_controls(&mut self, ctx: &Context) -> egui::Response {
		egui::TopBottomPanel::bottom("bottom_panel").resizable(true).show(ctx, |ui| {
			ui.vertical(|ui|{
				ui.horizontal_centered(|ui| {
					if ui.button("<<").clicked() {
						self.current_frame = 0;
					}
					if ui.button("<").clicked() || ctx.input(|i| i.key_released(egui::Key::A)) {
						self.current_frame = self.current_frame.saturating_sub(1);
					}
					if ui.button(">").clicked() || ctx.input(|i| i.key_released(egui::Key::D)) {
						self.current_frame = self.current_frame.saturating_add(1);
					}
					if ui.button(">>").clicked() {
						self.current_frame = 0;
					}
				});
			});
		}).response
	}
}

impl eframe::App for AutoRoto {
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		//eframe::set_value(storage, eframe::APP_KEY, self);
	}

	fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
		self.update_and_draw(ctx, frame);
	}
}

fn main() -> eframe::Result {
	env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([400.0, 300.0])
			.with_min_inner_size([300.0, 220.0]),
		/*
			.with_icon(
				eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
					.expect("Failed to load icon"),
			),
		*/
		..Default::default()
	};
	eframe::run_native(
		"AutoRoto",
		native_options,
		Box::new(|cc| Ok(Box::new(AutoRoto::new(cc)))),
	)
}
