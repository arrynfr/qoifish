use std::fs;
use std::fs::File;
use std::io::Read;

const RGB: u8 = 3;
const RGBA: u8 = 4;
const MAGIC: &[u8; 4] = b"qoif";
const QOI_OP_LUMA: u8 = 0b10_000000;
const QOI_OP_DIFF: u8 = 0b01_000000;
const QOI_OP_INDEX: u8 = 0b00_000000;
const QOI_OP_RUN: u8 = 0b11_000000;
const QOI_OP_RGB: u8 = 0b1111_1110;
const QOI_OP_RGBA: u8 = 0b1111_1111;
const OP_BITMASK: u8 = 0b11_000000;
const VAL_BITMASK: u8 = !OP_BITMASK;
const QOI_DIFF: std::ops::Range<u8> = 0..4;
const MAX_PIXELS_PER_RUN: u8 = 62;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
//const IMAGE: [u8; 10*10*4] = [253u8; 10*10*4];

fn qoi_calculate_index(r: u8, g: u8, b: u8, a: u8) -> usize {
	((r as u16 * 3 + 
	g as u16 * 5 +
	b as u16 * 7 + 
	a as u16 * 11)  % 64) as usize
}

fn encode(width: u32, height: u32) {
	let mut f = File::open("best_drawing.raw").unwrap();
	let metadata = fs::metadata("best_drawing.raw").unwrap();
	let mut IMAGE = vec![0; metadata.len() as usize];
	f.read(&mut IMAGE).unwrap();
	
	//Init image buffer
	let mut encoded_image: Vec<u8> = Vec::new();
	encoded_image.append(&mut MAGIC.to_vec());
	encoded_image.append(&mut width.to_be_bytes().to_vec());
	encoded_image.append(&mut height.to_be_bytes().to_vec());
	encoded_image.push(3);
	encoded_image.push(0);

	let mut previous_r: [u8; 64] = [0; 64];
	let mut previous_g: [u8; 64] = [0; 64];
	let mut previous_b: [u8; 64] = [0; 64];
	let mut previous_a: [u8; 64] = [255; 64];
	
	let (mut rp,mut gp,mut bp,mut ap) = (0u8,0u8,0u8,255u8);
	let (mut dr,mut dg,mut db) = (2u8,2u8,1u8);
	
	/*let qoi_op_diff: u8 = 	QOI_OP_DIFF | 
				(VAL_BITMASK & 
				((dr << 4) | 
				(dg << 2) | db));*/
	//let qoi_op_luma: (u8,u8) = 	(QOI_OP_LUMA | (VAL_BITMASK & dg), 
	//				(dr-dg << 4) | (db-dg));

	let mut previous_index: usize = 255;
	let mut qoi_run_pixels: u8 = 0;
	let size = width*height;
	for idx in 0..size {
		let idx = (idx*3) as usize;
		let rn = IMAGE[idx];
		let gn = IMAGE[idx+1];
		let bn = IMAGE[idx+2];
		/*let _an = IMAGE[idx+3];*/
		/*let rn = buffer.get(idx).unwrap();
		let gn = buffer.get(idx+1).unwrap();
		let bn = buffer.get(idx+2).unwrap();*/
		let an = 255u8;

		let index_position = qoi_calculate_index(rn,gn,bn,an);
		let previous_index = qoi_calculate_index(rp,gp,bp,ap);

		// Pixel is known?
		if 	rn == previous_r[index_position] &&
			gn == previous_g[index_position] &&
			bn == previous_b[index_position] &&
			an == previous_a[index_position] {
			
			// Pixel was same as last?
			if previous_index == index_position {
				if qoi_run_pixels < MAX_PIXELS_PER_RUN {qoi_run_pixels += 1;}
				else {
					encoded_image.push(QOI_OP_RUN | qoi_run_pixels-1);
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
		// Not known
		} else {
			// Update known pixel array
			previous_r[index_position] = rn;
			previous_g[index_position] = gn;
			previous_b[index_position] = bn;
			previous_a[index_position] = an;

			// check if we can diff encode
			dr = rp.wrapping_sub(rn);
			dr = (!dr+1).wrapping_add(2);
			dg = gp.wrapping_sub(gn);
			dg = (!dg+1).wrapping_add(2);
			db = bp.wrapping_sub(bn);
			db = (!db+1).wrapping_add(2);

			if 	QOI_DIFF.contains(&dr) &&
				QOI_DIFF.contains(&dg) &&
				QOI_DIFF.contains(&db) {
				encoded_image.push(QOI_OP_DIFF | dr << 4 | dg << 2 | db);
			} else if an != ap {
				encoded_image.append(&mut [QOI_OP_RGBA, rn, gn, bn, an].to_vec());
			} else {
				encoded_image.append(&mut [QOI_OP_RGB, rn, gn, bn].to_vec());
			}
		}
		(rp,gp,bp,ap) = (rn, gn, bn, an);
	}
	if qoi_run_pixels > 0 {encoded_image.push(QOI_OP_RUN | qoi_run_pixels-1);}
	let end: [u8; 8] = [0,0,0,0,0,0,0,1];
	encoded_image.append(&mut end.to_vec());
	println!("{:?}", encoded_image);
	fs::write("./image.qoi", encoded_image).expect("Unable to write file");
}

fn main() {
	encode(WIDTH, HEIGHT);
}
