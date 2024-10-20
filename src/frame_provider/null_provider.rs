
use image::RgbaImage;
use std::cell::LazyCell;

use crate::frame_provider::FrameProvider;

pub struct NullFrameProvider {
	cached_frame: RgbaImage
}

impl NullFrameProvider {
	pub fn new(width: u32, height: u32) -> Self {
		let mut imgbuf = image::ImageBuffer::new(width, height);

		// Iterate over the coordinates and pixels of the image
		for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
			let r = (0.3 * x as f32) as u8;
			let b = (0.3 * y as f32) as u8;
			*pixel = image::Rgba([r, 0, b, 255]);
		}

		Self {
			cached_frame: imgbuf
		}
	}
}

impl FrameProvider for NullFrameProvider {
	fn get_frame(&mut self, _frame_number: u64) -> RgbaImage {
		self.cached_frame.clone()
	}
}
