use std::{fs::{self, File}, io::{BufRead, BufReader, Write}};

const FONTS_DIR: &str = "./resources";
const OUTPUT_DIR: &str = "../../kernel/resources/fonts";


struct Font {
    family_name: String,
    number_chars: u32,
    font_bounding_box: (i32, i32, i32, i32),
    chars: Vec<Vec<u8>>
}

fn main() {
    let mut files: Vec<_> = fs::read_dir(FONTS_DIR)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    files.sort();

    if files.is_empty() {
        panic!("No fonts found");
    }

    fs::create_dir_all(OUTPUT_DIR).expect("Failed to create output directory");



    println!("{} fonts", files.len());
    for (index, path) in files.iter().enumerate() {
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);
        let mut font = Font {
            family_name: String::new(),
            number_chars: 0,
            font_bounding_box: (0, 0, 0, 0),
            chars: Vec::new()
        };

        let mut line_iterator = reader.lines();
        while let Some(line_result) = line_iterator.next() {
            let line = line_result.unwrap();
            let trimmed = line.trim();

            if trimmed.starts_with("FAMILY_NAME") {
                font.family_name = trimmed.split_whitespace().nth(1).unwrap_or("").trim_matches('"').into();
                continue;
            }

            if trimmed.starts_with("CHARS") {
                font.number_chars = trimmed.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                continue;
            }

            if trimmed.starts_with("FONTBOUNDINGBOX") {
                let coords: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
                if coords.len() == 4 {
                    font.font_bounding_box = (
                        coords[0].parse().unwrap_or(0),
                        coords[1].parse().unwrap_or(0),
                        coords[2].parse().unwrap_or(0),
                        coords[3].parse().unwrap_or(0)
                    );
                }
                continue;
            }

            if trimmed.starts_with("STARTCHAR") {
                let mut c = Vec::new();

                for char_line_result in line_iterator.by_ref() {
                    let char_line = char_line_result.unwrap();
                    let char_trimmed = char_line.trim();

                    if char_trimmed.starts_with("BITMAP") {
                        for bitmap_line_result in line_iterator.by_ref() {
                            let bitmap_line = bitmap_line_result.unwrap();
                            let bitmap_trimmed = bitmap_line.trim();

                            if bitmap_trimmed.starts_with("ENDCHAR") {
                                break;
                            }

                            if let Ok(byte) = u8::from_str_radix(bitmap_trimmed, 16) {
                                c.push(byte);
                            }
                        }
                        break;
                    }
                }

                font.chars.push(c);
                continue;
            }
        }

        println!("{}/{} | Font: {}, chars: {}, font_bounding_box: {:?}", index + 1, files.len(), font.family_name, font.number_chars, font.font_bounding_box);



        let mut output_file = File::create(
            format!("{}/{}-{}x{}.bin", OUTPUT_DIR, font.family_name.to_lowercase(), font.font_bounding_box.0, font.font_bounding_box.1)
        ).unwrap();

        let mut name_bytes = [0u8; 32];
        let name_src = font.family_name.as_bytes();

        let name_len = name_src.len().min(32);
        name_bytes[..name_len].copy_from_slice(&name_src[..name_len]);

        output_file.write_all(&name_bytes).unwrap();
        output_file.write_all(&font.number_chars.to_le_bytes()).unwrap();

        output_file.write_all(&font.font_bounding_box.0.to_le_bytes()).unwrap();
        output_file.write_all(&font.font_bounding_box.1.to_le_bytes()).unwrap();
        output_file.write_all(&font.font_bounding_box.2.to_le_bytes()).unwrap();
        output_file.write_all(&font.font_bounding_box.3.to_le_bytes()).unwrap();

        for ch in &font.chars {
            if ch.len() == font.font_bounding_box.1 as usize {
                output_file.write_all(&ch).unwrap();

            } else if ch.len() < font.font_bounding_box.1 as usize {
                let mut bitmap_padded = ch.clone();
                bitmap_padded.resize(font.font_bounding_box.1 as usize, 0x00);
                output_file.write_all(&bitmap_padded).unwrap();

            } else {
                output_file.write_all(&ch[..font.font_bounding_box.1 as usize]).unwrap();
            }
        }

        output_file.flush().unwrap();
    }
}
