use glob::glob;
use image::{ImageReader, RgbImage};
use rfd::FileDialog;
use std::path::PathBuf;

// This is going to have to become a trait or something.
pub enum FrameProvider {
	NullFrameProvider,
	DirectoryFrameProvider(Vec<PathBuf>),
	MovieFrameProvider(PathBuf),
}

pub fn get_frame_provider(file_sequence: bool) -> Option<FrameProvider> {
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

pub fn get_frame(fp: &FrameProvider, frame_number: u64) -> RgbImage {
	match fp {
		FrameProvider::NullFrameProvider => {
			let mut imgbuf = image::ImageBuffer::new(640, 480);

			// Iterate over the coordinates and pixels of the image
			for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
				let r = (0.3 * x as f32) as u8;
				let b = (0.3 * y as f32) as u8;
				*pixel = image::Rgb([r, 0, b]);
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
			let img = ImageReader::open(&file_list[frame_number as usize]).expect("Frame with file {} could not be opened. Missing?").decode()?;
			//let img2 = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?;
			img.into_rgb8()
		},
		FrameProvider::MovieFrameProvider(movie_filename) => {
			todo!()
		},
	}
}