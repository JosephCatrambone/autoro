use std::cell::LazyCell;
use glob::glob;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, RgbaImage, RgbImage};
use rfd::FileDialog;
use std::path::PathBuf;
use eframe::epaint::{Color32, ColorImage};

mod image_sequence_provider;
mod null_provider;
mod video_provider;

pub use null_provider::NullFrameProvider;
pub use image_sequence_provider::ImageSequenceFrameProvider;

pub trait FrameProvider {
	fn get_frame(&mut self, frame_number: u64) -> RgbaImage;
}

//	DirectoryFrameProvider(Vec<PathBuf>),
//	MovieFrameProvider(PathBuf),


pub fn get_frame_provider(file_sequence: bool) -> Option<Box<dyn FrameProvider>> {
	/*
	let mut files = if file_sequence {
		FileDialog::new()
			.add_filter("video file", &["mp4", "webm"])
			.set_directory("/")
			.pick_file();
	}
	*/

	if file_sequence {
		if let Some(files) = FileDialog::new().pick_files() {
			return Some(Box::new(ImageSequenceFrameProvider::new(files)));
		}
	} else {
		if let Some(file) = FileDialog::new().pick_file() {
			todo!()
			//return Some(Box::new(FrameProvider::MovieFrameProvider(file)));
		}
	};

	None
}

pub fn image_to_egui_image(frame: &RgbaImage) -> ColorImage {
	let size = [frame.width() as usize, frame.height() as usize];
	let mut pixels = Vec::with_capacity((3*frame.width()*frame.height()) as usize);
	for p in frame.pixels() {
		pixels.push(Color32::from_rgba_unmultiplied(p.0[0], p.0[1], p.0[2], p.0[3]));
	}
	ColorImage { size, pixels }
}

