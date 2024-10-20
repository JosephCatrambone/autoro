use crate::frame_provider::FrameProvider;
use image::{ImageReader, RgbaImage};
use std::path::PathBuf;

fn load_image_or_make_default(path: &PathBuf) -> RgbaImage {
	let img = ImageReader::open(path)
		.expect(&format!("Frame with file {} could not be opened. Missing?", path.to_string_lossy()))
		.decode()
		.expect("Failed to load image.");
	img.into_rgba8()
}

pub struct ImageSequenceFrameProvider {
	file_list: Vec<PathBuf>,
	cached_frame: RgbaImage,
	last_loaded_frame: u64,
}

impl ImageSequenceFrameProvider {
	pub(crate) fn new(frames: Vec<PathBuf>) -> Self {
		let first_frame = load_image_or_make_default(&frames[0]);
		Self {
			file_list: frames,
			cached_frame: first_frame,
			last_loaded_frame: 0
		}
	}
}

impl FrameProvider for ImageSequenceFrameProvider {
	fn get_frame(&mut self, frame_number: u64) -> RgbaImage {
		/*
			for entry in dir_glob.expect("Failed to read glob pattern") {
				match entry {
					Ok(path) => println!("{:?}", path.display()),
					Err(e) => println!("{:?}", e),
				}
			}
			*/
		if frame_number != self.last_loaded_frame {
			//let img2 = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?;
			self.cached_frame = load_image_or_make_default(&self.file_list[frame_number as usize]);
			self.last_loaded_frame = frame_number;
		}
		self.cached_frame.clone()
	}
}