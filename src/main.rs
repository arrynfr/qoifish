use std::fs;
use std::fs::File;
use std::io::Read;
use std::env;

const MAGIC: &[u8; 4] = b"qoif";
const RGB: u8 = 3;
const _RGBA: u8 = 4;
const _QOI_OP_LUMA: u8 = 0b10_000000;
const QOI_OP_DIFF: u8 = 0b01_000000;
const QOI_OP_INDEX: u8 = 0b00_000000;
const QOI_OP_RUN: u8 = 0b11_000000;
const QOI_OP_RGB: u8 = 0b1111_1110;
const QOI_OP_RGBA: u8 = 0b1111_1111;
const QOI_DIFF: std::ops::Range<u8> = 0..4;
const MAX_PIXELS_PER_RUN: u8 = 62;

#[derive(Debug, Copy, Clone, PartialEq)]
struct Pixel(u8,u8,u8,u8);

fn qoi_calculate_index(p: Pixel) -> usize {
	((p.0 as u16 * 3 + 
	p.1 as u16 * 5 +
	p.2 as u16 * 7 + 
	p.3 as u16 * 11)  % 64) as usize
}

fn encode(width: u32, height: u32, file_path: &str) {
	let mut f = File::open(file_path).unwrap();
	let metadata = fs::metadata(file_path).unwrap();
	let mut image = vec![0; metadata.len() as usize];
	f.read(&mut image).unwrap();
	
	//println!("{:?}", image);
	//Init image buffer
	let mut encoded_image: Vec<u8> = Vec::new();
	encoded_image.append(&mut MAGIC.to_vec());
	encoded_image.append(&mut width.to_be_bytes().to_vec());
	encoded_image.append(&mut height.to_be_bytes().to_vec());
	encoded_image.push(RGB);
	encoded_image.push(0);
	
	let mut previous_pixel: [Pixel; 64] = [Pixel(0,0,0,255); 64];
	let mut pp = Pixel(0u8,0u8,0u8,255u8);
	let (mut dr,mut dg,mut db);
	let mut qoi_run_pixels: u8 = 0;
	let size = metadata.len()/3;
	
	for idx in 0..size {
		let idx = (idx*3) as usize;
		let curr_pixel = Pixel(	image[idx], image[idx+1],
					image[idx+2], 255u8);

		let index_position = qoi_calculate_index(curr_pixel);
		let previous_index = qoi_calculate_index(pp);
		
		// Pixel is known?
		if curr_pixel == previous_pixel[index_position] {
			if previous_index == index_position {
				if qoi_run_pixels < MAX_PIXELS_PER_RUN {
					qoi_run_pixels += 1;
				} else {
					encoded_image.push(QOI_OP_RUN 
							| qoi_run_pixels-1);
					qoi_run_pixels = 1;
				}
			// Known but not same as last
			} else {
				if qoi_run_pixels > 0 { 
					encoded_image.push(QOI_OP_RUN | qoi_run_pixels-1);
					qoi_run_pixels = 0;
				}
				encoded_image.push(QOI_OP_INDEX | index_position as u8);
			}
		} else {
			if qoi_run_pixels > 0 { 
				encoded_image.push(QOI_OP_RUN | qoi_run_pixels-1);
				qoi_run_pixels = 0;
			}
			// Update known pixel array
			previous_pixel[index_position] = curr_pixel; 

			// check if we can diff encode
			dr = pp.0.wrapping_sub(curr_pixel.0);
			dr = (!dr+1).wrapping_add(2);
			dg = pp.1.wrapping_sub(curr_pixel.1);
			dg = (!dg+1).wrapping_add(2);
			db = pp.2.wrapping_sub(curr_pixel.2);
			db = (!db+1).wrapping_add(2);
			
			if 	QOI_DIFF.contains(&dr) &&
				QOI_DIFF.contains(&dg) &&
				QOI_DIFF.contains(&db) {
				encoded_image.push(QOI_OP_DIFF | ((dr << 4)
							| (dg << 2) | db));
			} else if curr_pixel.3 != pp.3 {
				encoded_image.append(&mut [QOI_OP_RGBA, curr_pixel.0,
							curr_pixel.1, curr_pixel.2,
							curr_pixel.3].to_vec());
			} else {
				encoded_image.append(&mut [QOI_OP_RGB, curr_pixel.0, 
							curr_pixel.1, curr_pixel.2].to_vec());
			}
		}
		pp = curr_pixel;
	}
	if qoi_run_pixels > 0 {encoded_image.push(QOI_OP_RUN | qoi_run_pixels-1);}
	let end: [u8; 8] = [0,0,0,0,0,0,0,1];
	encoded_image.append(&mut end.to_vec());
	fs::write("./image.qoi", encoded_image).expect("Unable to write file");
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let file_path = &args[1];
	let (width,height) = (	args[2].parse::<u32>().unwrap(),
				args[3].parse::<u32>().unwrap());
	encode(width, height, file_path);
}
