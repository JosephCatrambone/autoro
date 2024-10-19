#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::cell::Cell;
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
mod frame_provider;

use egui::{emath, vec2, Color32, ColorImage, Context, Frame, Image, Pos2, Rect, Sense, Stroke, Ui, Window, TextureFilter, TextureHandle, TextureOptions};
use image::{ImageBuffer, RgbImage, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;

use frame_provider::{FrameProvider, get_frame, get_frame_provider, image_to_egui_image};

// Huge props to egui-video which I could not use but was instrumental to figuring this out.
// Look at https://github.com/n00kii/egui-video/blob/main/src/lib.rs for overlap.

//#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[cfg_attr(feature = "serde", serde(default))]
pub struct AutoRoto {
	points: HashMap<usize, Vec<Pos2>>,  // frame number to list of positions in 0-1 normalized form.
	lines: Vec<Vec<Pos2>>, 	// in 0-1 normalized coordinates
	stroke: Stroke,

	frame_provider: FrameProvider,
	cached_frames: HashMap<usize, image::RgbaImage>,
	current_frame: usize,

	display_texture_options: TextureOptions,
	display_texture_handle: TextureHandle,
	display_texture_frame: usize, // The current image frame loaded into the texture.
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
			points: Default::default(),
			lines: Default::default(),
			stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
			frame_provider: FrameProvider::NullFrameProvider,
			cached_frames: Default::default(),
			current_frame: 0,
			display_texture_handle: cc.egui_ctx.load_texture("display_image", ColorImage::example(), TextureOptions::default()),
			display_texture_options: TextureOptions::default(),
			display_texture_frame: 0,
		}
	}

	/// Create the [`egui::Image`] for the video frame.
	fn generate_frame_image(&mut self, size: egui::Vec2) -> Image {
		if self.current_frame != self.display_texture_frame {
			if !self.cached_frames.contains_key(&self.current_frame) {
				// insert
			}

			let cached_frame_ref = self.cached_frames.get(&self.current_frame).expect("CANNOT FETCH BACK FRAME JUST ADDED TO LIST.");

			self.display_texture_handle.set(
				image_to_egui_image(cached_frame_ref),
				self.display_texture_options
			);
			self.display_texture_frame = self.current_frame;
		}

		Image::new(egui::load::SizedTexture::new(self.display_texture_handle.id(), size)).sense(Sense::click())
	}

	/// Draw the video frame with a specific rect (without controls).
	fn render_frame(&mut self, ui: &mut Ui, size: egui::Vec2) -> egui::Response {
		ui.add(self.generate_frame_image(size))
	}

	/// Draw the video frame (without controls).
	fn render_frame_at(&mut self, ui: &mut Ui, rect: Rect) -> egui::Response {
		ui.put(rect, self.generate_frame_image(rect.size()))
	}

	// Draw the current frame and all the points on top.
	fn ui_current_frame(&mut self, ui: &mut Ui) -> egui::Response {
		let (mut response, painter) =
			ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

		response.mark_changed();
		if !self.cached_frames.contains_key(&self.current_frame) {
		}

		//painter.image(self.display_texture_handle.id(), Rect {}, Rect::, Default::default());
		self.render_frame(ui, egui::Vec2::new(0.0, 0.0));

		response
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

	pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
		let (mut response, painter) =
			ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

		let to_screen = emath::RectTransform::from_to(
			Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
			response.rect,
		);
		let from_screen = to_screen.inverse();

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

	fn draw_ui(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:
			egui::menu::bar(ui, |ui| {
				let is_web = cfg!(target_arch = "wasm32");
				ui.menu_button("File", |ui| {
					if ui.button("Open Image Sequence").clicked() {
						if let Some(files) = rfd::FileDialog::new().pick_files() {
							self.frame_provider = FrameProvider::DirectoryFrameProvider(files);
							self.current_frame = 0;
							self.cached_frames.clear();
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
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			// The central panel the region left after adding TopPanel's and SidePanel's
			ui.heading("eframe template");

			ui.horizontal(|ui| {
				ui.label("Write something: ");
				//ui.text_edit_singleline(&mut self.label);
			});


			ui.vertical_centered(|ui| {
				//ui.add(crate::egui_github_link_file!());
			});
			self.ui_control(ui);
			ui.label("Paint with your mouse/touch!");
			Frame::canvas(ui.style()).show(ui, |ui| {
				self.ui_content(ui);
			});
			//ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
			//if ui.button("Increment").clicked() {
			//	self.value += 1.0;
			//}

			ui.separator();

			ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				//powered_by_egui_and_eframe(ui);
				egui::warn_if_debug_build(ui);
			});
		});
	}
}

impl eframe::App for AutoRoto {
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		//eframe::set_value(storage, eframe::APP_KEY, self);
	}

	fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
		self.draw_ui(ctx, frame);
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
