use std::{fs::{self, File}, io::{BufWriter, Write}};

use image::ImageReader;

const FRAMES_DIR: &str = "./resources";
const OUTPUT_DIR: &str = "../../kernel/resources";

fn main() {
    let mut files: Vec<_> = fs::read_dir(FRAMES_DIR)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    files.sort();

    if files.is_empty() {
        panic!("No pictures found in {}", FRAMES_DIR);
    }

    println!("{} pictures found", files.len());



    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");

    for (index, path) in files.iter().enumerate() {
        if !path.is_file() || (
            path.extension().unwrap_or_default() != "png" && 
            path.extension().unwrap_or_default() != "jpg" && 
            path.extension().unwrap_or_default() != "jpeg"
        ) {
            continue;
        }

        let img = ImageReader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        let output = File::create(format!("{}/{}.bin", OUTPUT_DIR, path.file_stem().unwrap().to_str().unwrap())).unwrap();
        let mut writer = BufWriter::new(output);


        let width = img.width() as u16;
        let height = img.height() as u16;

        writer.write_all(&width.to_le_bytes()).unwrap();
        writer.write_all(&height.to_le_bytes()).unwrap();

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x.into(), y.into());
                let [r, g, b, a] = pixel.0;

                writer.write_all(&[r, g, b, a]).unwrap();
            }
        }


        println!("Processed frame {}/{}", index + 1, files.len());
        
        writer.flush().unwrap();
    }
}
