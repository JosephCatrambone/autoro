#![deny(clippy::all)]
#![forbid(unsafe_code)]

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

mod frame_provider;
use frame_provider::FrameProvider;

use image::{ImageBuffer, RgbImage, RgbaImage};
use egui::{emath, vec2, Color32, Context, Frame, Pos2, Rect, Sense, Stroke, Ui, Window};
use serde::{Deserialize, Serialize};

//#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
//#[cfg_attr(feature = "serde", serde(default))]
#[derive(Serialize, Deserialize)]
pub struct AutoRoto {
	lines: Vec<Vec<Pos2>>, 	// in 0-1 normalized coordinates
	stroke: Stroke,
}

impl Default for AutoRoto {
	fn default() -> Self {
		Self {
			lines: Default::default(),
			stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
		}
	}
}

impl AutoRoto {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Default::default()
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

	fn ui(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			//ui.add(crate::egui_github_link_file!());
		});
		self.ui_control(ui);
		ui.label("Paint with your mouse/touch!");
		Frame::canvas(ui.style()).show(ui, |ui| {
			self.ui_content(ui);
		});
	}
}

impl eframe::App for AutoRoto {
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:

			egui::menu::bar(ui, |ui| {
				// NOTE: no File->Quit on web pages!
				let is_web = cfg!(target_arch = "wasm32");
				if !is_web {
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
					});
					ui.add_space(16.0);
				}

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

			self.ui(ui);

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
