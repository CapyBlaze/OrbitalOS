use std::{fs::{self, File}, io::{BufWriter, Write}};

use image::ImageReader;

const FRAMES_DIR: &str = "./resources/frames";
const OUTPUT_DIR: &str = "../../kernel/resources";

fn main() {
    let mut files: Vec<_> = fs::read_dir(FRAMES_DIR)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    files.sort();

    if files.is_empty() {
        panic!("No frames found");
    }

    let first = ImageReader::open(&files[0])
        .unwrap()
        .decode()
        .unwrap()
        .to_luma8();

    let width = first.width();
    let height = first.height();

    println!("{}x{}", width, height);
    println!("{} frames", files.len());


    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");

    let output = File::create(format!("{}/bad_apple.bin", OUTPUT_DIR)).unwrap();
    let mut writer = BufWriter::new(output);

    let header_len = 12usize;
    let header_padding = (512 - (header_len % 512)) % 512;
    let frame_size = (width as usize) * (height as usize) / 8;
    let frame_padded_size = ((frame_size + 511) / 512) * 512;

    writer.write_all(&width.to_le_bytes()).unwrap();
    writer.write_all(&height.to_le_bytes()).unwrap();
    writer.write_all(&(files.len() as u32).to_le_bytes()).unwrap();
    if header_padding != 0 {
        writer.write_all(&vec![0u8; header_padding]).unwrap();
    }

    for (index, path) in files.iter().enumerate() {
        let img = ImageReader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .to_luma8();

        let mut current_byte: u8 = 0;
        let mut bit_count = 0;

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y)[0];
                let bit = if pixel > 128 { 1 } else { 0 };

                current_byte <<= 1;
                current_byte |= bit;

                bit_count += 1;

                if bit_count == 8 {
                    writer.write_all(&[current_byte]).unwrap();

                    current_byte = 0;
                    bit_count = 0;
                }
            }
        }

        if bit_count != 0 {
            current_byte <<= 8 - bit_count;
            writer.write_all(&[current_byte]).unwrap();
        }

        if frame_padded_size > frame_size {
            writer.write_all(&vec![0u8; frame_padded_size - frame_size]).unwrap();
        }

        println!("Processed frame {}/{}", index + 1, files.len());
    }

    writer.flush().unwrap();
}
