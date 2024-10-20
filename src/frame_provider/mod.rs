use std::cell::LazyCell;
use glob::glob;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, RgbaImage, RgbImage};
use rfd::FileDialog;
use std::path::PathBuf;
use eframe::epaint::{Color32, ColorImage};

pub trait FrameProvider {
	fn get_frame(&mut self, frame_number: u64) -> RgbaImage;
}

pub struct NullFrameProvider {
	cached_frame: LazyCell<RgbaImage>
}

impl FrameProvider for NullFrameProvider {
	fn get_frame(&mut self, frame_number: u64) -> RgbaImage {
		let mut imgbuf = image::ImageBuffer::new(640, 480);

		// Iterate over the coordinates and pixels of the image
		for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
			let r = (0.3 * x as f32) as u8;
			let b = (0.3 * y as f32) as u8;
			*pixel = image::Rgba([r, 0, b, 255]);
		}

		imgbuf
	}
}

//	DirectoryFrameProvider(Vec<PathBuf>),
//	MovieFrameProvider(PathBuf),


pub fn get_frame_provider(file_sequence: bool) -> Option<dyn FrameProvider> {
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
			return Some(FrameProvider::DirectoryFrameProvider(files));
		}
	} else {
		if let Some(file) = FileDialog::new().pick_file() {
			return Some(FrameProvider::MovieFrameProvider(file));
		}
	};

	None
}

pub fn get_frame(fp: &FrameProvider, frame_number: u64) -> RgbaImage {
	match fp {
		FrameProvider::NullFrameProvider => {
			let mut imgbuf = image::ImageBuffer::new(640, 480);

			// Iterate over the coordinates and pixels of the image
			for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
				let r = (0.3 * x as f32) as u8;
				let b = (0.3 * y as f32) as u8;
				*pixel = image::Rgba([r, 0, b, 255]);
			}

			imgbuf
		},
		FrameProvider::DirectoryFrameProvider(file_list) => {
			/*
			for entry in dir_glob.expect("Failed to read glob pattern") {
				match entry {
					Ok(path) => println!("{:?}", path.display()),
					Err(e) => println!("{:?}", e),
				}
			}
			*/
			let img = ImageReader::open(&file_list[frame_number as usize]).expect("Frame with file {} could not be opened. Missing?").decode().expect("Failed to load image.");
			//let img2 = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?;
			img.into_rgba8()
		},
		FrameProvider::MovieFrameProvider(movie_filename) => {
			todo!()
		},
	}
}

pub fn image_to_egui_image(frame: &RgbaImage) -> ColorImage {
	let size = [frame.width() as usize, frame.height() as usize];
	let mut pixels = Vec::with_capacity((3*frame.width()*frame.height()) as usize);
	for p in frame.pixels() {
		pixels.push(Color32::from_rgba_unmultiplied(p.0[0], p.0[1], p.0[2], p.0[3]));
	}
	ColorImage { size, pixels }
}

