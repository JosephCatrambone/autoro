use crate::frame_provider::FrameProvider;
use image::{ImageReader, RgbaImage};
use ndarray::{Axis, Array3};
use std::path::PathBuf;
use video_rs::decode::Decoder;

pub struct VideoFrameProvider {
	video_decode_stream: Decoder,
	cached_frame: RgbaImage,
	width: u64,
	height: u64,
	last_loaded_frame: usize,
	num_frames: usize,
}

impl VideoFrameProvider {
	pub(crate) fn new(file: PathBuf) -> Self {
		let mut video = Decoder::new(file).expect("failed to create decoder");
		let (video_width, video_height) = video.size();

		let img = RgbaImage::new(video_width, video_height);
		let last_frame = video.frames().unwrap() as usize;

		println!("Loaded video with size {}x{}, {} frames.", &video_width, &video_height, &last_frame);

		Self {
			video_decode_stream: video,
			cached_frame: img,
			width: video_width as u64,
			height: video_height as u64,
			last_loaded_frame: last_frame+1,
			num_frames: last_frame,
		}
	}
}

impl FrameProvider for VideoFrameProvider {
	fn get_frame(&mut self, mut frame_number: usize) -> RgbaImage {
		let frame_number: usize = frame_number.min(self.get_num_frames()-1);

		// Use cached if we can...
		if frame_number == self.last_loaded_frame {
			return self.cached_frame.clone();
		}

		if frame_number == 0 {
			self.last_loaded_frame = 0;
			self.video_decode_stream.seek_to_start().expect("Failed to seek to the start of the stream.");
		} else if frame_number == self.last_loaded_frame + 1 {
			self.last_loaded_frame += 1;
			// Don't need to seek in this case.
		} else {
			self.last_loaded_frame = frame_number;
			// TODO: This seek isn't working when we set it to be the last frame.
			self.video_decode_stream.seek_to_frame(self.last_loaded_frame as i64).expect("Unable to seek to frame.");
		}

		println!("Fetching frame {}", &self.last_loaded_frame);
		let (ts, frame) = self.video_decode_stream.decode().expect("Failed to decode frame from video stream.");
		let shape = frame.shape().to_owned();
		let mut image = RgbaImage::new(self.width as u32, self.height as u32);
		/*
		let mut arr: Array2<()> = array![[(), ()], [(), ()], [(), ()]];
		arr.slice_axis_inplace(Axis(0), (1..).into());

		let shape = arr.shape().to_owned();
		let strides = arr.strides().to_owned();
		let (v, offset) = arr.into_raw_vec_and_offset();

		assert_eq!(v, &[(), (), (), (), (), ()]);
		for row in 0..shape[0] {
			for col in 0..shape[1] {
				let index = (
					offset.unwrap() as isize
					+ row as isize * strides[0]
					+ col as isize * strides[1]
				) as usize;
				assert_eq!(v[index], ());
			}
		}
		*/
		assert_eq!(self.width as usize, shape[1]);
		assert_eq!(self.height as usize, shape[0]);
		for (x, y, p) in image.enumerate_pixels_mut() {
			p[0] = *frame.get((y as usize, x as usize, 0)).unwrap();
			p[1] = *frame.get((y as usize, x as usize, 1)).unwrap();
			p[2] = *frame.get((y as usize, x as usize, 2)).unwrap();
			p[3] = 255;
		}
		self.cached_frame = image.clone();
		image
	}

	fn get_num_frames(&self) -> usize {
		self.num_frames
	}
}