use core::{ptr::null_mut, slice, str};
use alloc::{vec::Vec, vec};
use spin::Mutex;
use core::convert::TryInto;

use crate::boot_info;

#[derive(Clone, Copy)]
pub struct ColorRGB {
    r: u8,
    g: u8,
    b: u8,
}

impl ColorRGB {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

}



#[repr(C)]
pub struct FrameBuffer {
    pub buffer_ptr: *mut u8,
    pub buffer_size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
}

unsafe impl Send for FrameBuffer {}
unsafe impl Sync for FrameBuffer {}


struct FrameBufferState {
    buffer: &'static mut [u8],
    backbuffer: Vec<u8>,
    stride_bytes: usize,
    width: usize,
    height: usize,
    bytes_per_pixel: usize,
}

static FB_STATE: Mutex<Option<FrameBufferState>> = Mutex::new(None);
pub static FRAMEBUFFER: Mutex<FrameBuffer> = Mutex::new(FrameBuffer {
    buffer_ptr: null_mut(),
    buffer_size: 0,
    width: 0,
    height: 0,
    stride: 0,
});


pub static FONT_MANAGER: Mutex<Vec<KernelFont>> = Mutex::new(Vec::new());
pub struct KernelFont {
    pub name: FontName,
    pub nb_chars: usize,
    pub font_bounding_box: (usize, usize, i32, i32),
    pub char_size: usize,
    pub data: Vec<u8>,
}

impl KernelFont {
    pub fn new(name: FontName, bytes: Vec<u8>) -> Self {
        let nb_chars = i32::from_le_bytes(bytes
            .get(32..36)
            .unwrap()
            .try_into()
            .unwrap()
        ) as usize;

        let width = i32::from_le_bytes(bytes
            .get(36..40)
            .unwrap()
            .try_into()
            .unwrap()
        ) as usize;

        let height = i32::from_le_bytes(bytes
            .get(40..44)
            .unwrap()
            .try_into()
            .unwrap()
        ) as usize;

        let offset_x = i32::from_le_bytes(bytes
            .get(44..48)
            .unwrap()
            .try_into()
            .unwrap()
        );

        let offset_y = i32::from_le_bytes(bytes
            .get(48..52)
            .unwrap()
            .try_into()
            .unwrap()
        );


        let font_bounding_box = (width, height, offset_x, offset_y);

        let bytes_per_row = (width + 7) / 8;
        let char_size = bytes_per_row * height;


        Self {
            name,
            nb_chars,
            font_bounding_box,
            char_size,
            data: bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontName {
    SpleenSmallSmall,
    SpleenSmall,
    SpleenLarge,
    SpleenBig,
    SpleenBigBig,
    Unknown,
}

impl FontName {
    pub fn as_str(&self) -> &'static str {
        match self {
            FontName::SpleenSmallSmall => "Spleen Small Small",
            FontName::SpleenSmall => "Spleen Small",
            FontName::SpleenLarge => "Spleen Large",
            FontName::SpleenBig => "Spleen Big",
            FontName::SpleenBigBig => "Spleen Big Big",
            FontName::Unknown => "Unknown",
        }
    }
}


pub unsafe fn init(vbe_info: *const u8) {
    let pitch  = *(vbe_info.add(16) as *const u16) as usize;
    let width  = *(vbe_info.add(18) as *const u16) as usize;
    let height = *(vbe_info.add(20) as *const u16) as usize;
    let bpp    = *(vbe_info.add(25) as *const u8) as usize;
    let fb_addr = *(vbe_info.add(40) as *const u32);
    let fb_ptr = fb_addr as *mut u8;
    let bytes_per_pixel = (bpp + 7) / 8;


    let mut fb = FRAMEBUFFER.lock();
    fb.buffer_ptr = fb_ptr;
    fb.width = width;
    fb.height = height;
    fb.stride = pitch;
    fb.buffer_size = pitch * height;


    let buffer_slice = unsafe {
        slice::from_raw_parts_mut(fb.buffer_ptr, fb.buffer_size)
    };
    let buffer_size = fb.buffer_size;
    let backbuffer = vec![0u8; buffer_size];


    let mut guard = FB_STATE.lock();
    *guard = Some(FrameBufferState {
        buffer: buffer_slice,
        backbuffer,
        stride_bytes: fb.stride,
        width: fb.width,
        height: fb.height,
        bytes_per_pixel,
    });
}

pub fn init_fonts() {
    let mut manager = FONT_MANAGER.lock();

    manager.clear();

    if let Some(bytes) = boot_info::load_file("spleen-6x12.bin") {
        manager.push(KernelFont::new(FontName::SpleenSmallSmall, bytes));
    }
    

    if let Some(bytes) = boot_info::load_file("spleen-8x16.bin") {
        manager.push(KernelFont::new(FontName::SpleenSmall, bytes));
    }

    if let Some(bytes) = boot_info::load_file("spleen-12x24.bin") {
        manager.push(KernelFont::new(FontName::SpleenLarge, bytes));
    }

    if let Some(bytes) = boot_info::load_file("spleen-16x32.bin") {
        manager.push(KernelFont::new(FontName::SpleenBig, bytes));
    }

    if let Some(bytes) = boot_info::load_file("spleen-32x64.bin") {
        manager.push(KernelFont::new(FontName::SpleenBigBig, bytes));
    }
}

pub fn put_pixel(x: usize, y: usize, color: ColorRGB) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        if x >= state.width || y >= state.height {
            return;
        }
        
        let i = y * state.stride_bytes + x * state.bytes_per_pixel;

        if i + state.bytes_per_pixel <= state.backbuffer.as_slice().len() {
            state.backbuffer.as_mut_slice()[i] = color.b;
            if state.bytes_per_pixel > 1 {
                state.backbuffer.as_mut_slice()[i + 1] = color.g;
            }
            if state.bytes_per_pixel > 2 {
                state.backbuffer.as_mut_slice()[i + 2] = color.r;
            }
            if state.bytes_per_pixel > 3 {
                state.backbuffer.as_mut_slice()[i + 3] = 0x00;
            }
        }
    }
}

pub fn draw_bitmap_1bpp (
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    bitmap: &[u8],
    fg: ColorRGB,
    bg: ColorRGB,
) {
    let mut fb = FB_STATE.lock();
    let state = match fb.as_mut() {
        Some(s) => s,
        None => return,
    };


    let bytes_per_row = (width + 7) / 8;
    
    for py in 0..height {
        for px in 0..width {
            let byte_index = py * bytes_per_row + (px / 8);

            let byte = bitmap[byte_index];
            let bit = (byte >> (7 - (px % 8))) & 1;

            let color = if bit == 1 { fg } else { bg };

            let sx = x + px;
            let sy = y + py;

            let i =
                sy * state.stride_bytes +
                sx * state.bytes_per_pixel;

            if i + 3 >= state.backbuffer.as_slice().len() {
                continue;
            }

            state.backbuffer.as_mut_slice()[i + 0] = color.b;
            state.backbuffer.as_mut_slice()[i + 1] = color.g;
            state.backbuffer.as_mut_slice()[i + 2] = color.r;

            if state.bytes_per_pixel == 4 {
                state.backbuffer.as_mut_slice()[i + 3] = 0;
            }
        }
    }
}

pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: ColorRGB) {
    let mut guard = FB_STATE.lock();
    let state = match guard.as_mut() {
        Some(s) => s,
        None => return,
    };

    for py in 0..height {
        for px in 0..width {
            let sx = x + px;
            let sy = y + py;
            if sx >= state.width || sy >= state.height { continue; }

            let i = sy * state.stride_bytes + sx * state.bytes_per_pixel;
            if i + state.bytes_per_pixel > state.backbuffer.len() { continue; }

            state.backbuffer.as_mut_slice()[i]     = color.b;
            state.backbuffer.as_mut_slice()[i + 1] = color.g;
            state.backbuffer.as_mut_slice()[i + 2] = color.r;
            if state.bytes_per_pixel == 4 {
                state.backbuffer.as_mut_slice()[i + 3] = 0x00;
            }
        }
    }
}

pub fn clear(color: ColorRGB) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        for y in 0..state.height {
            let row = y * state.stride_bytes;

            for x in 0..state.width {
                let i = row + x * state.bytes_per_pixel;

                state.backbuffer.as_mut_slice()[i] = color.b;
                if state.bytes_per_pixel > 1 {
                    state.backbuffer.as_mut_slice()[i + 1] = color.g;
                }
                if state.bytes_per_pixel > 2 {
                    state.backbuffer.as_mut_slice()[i + 2] = color.r;
                }
                if state.bytes_per_pixel > 3 {
                    state.backbuffer.as_mut_slice()[i + 3] = 0x00;
                }
            }
        }
    }
}

pub fn text_draw(x: usize, y: usize, text: &str, font_name: FontName, fg: ColorRGB, bg: ColorRGB) {
    let glyphs: alloc::vec::Vec<_> = {
        let manager = FONT_MANAGER.lock();
        let Some(font) = manager.iter().find(|f| f.name == font_name) else { return; };
        text.chars()
            .filter(|c| *c as usize >= 32)
            .filter_map(|c| {
                let ascii = c as usize;
                let start = 52 + (ascii - 32) * font.char_size;
                let end = start + font.char_size;
                let bytes = font.data.get(start..end)?.to_vec();
                Some((font.font_bounding_box, bytes))
            })
            .collect()
    };


    let mut guard = FB_STATE.lock();
    let state = match guard.as_mut() { Some(s) => s, None => return };

    let mut current_x = x;
    for (bbox, glyph_bytes) in &glyphs {
        let (font_width, font_height, _, _) = *bbox;
        let bytes_per_row = (font_width + 7) / 8;

        for py in 0..font_height {
            for px in 0..font_width {
                let byte_index = py * bytes_per_row + (px / 8);
                let byte = glyph_bytes[byte_index];
                let bit = (byte >> (7 - (px % 8))) & 1;
                let color = if bit == 1 { fg } else { bg };

                let sx = current_x + px;
                let sy = y + py;
                if sx >= state.width || sy >= state.height { continue; }

                let i = sy * state.stride_bytes + sx * state.bytes_per_pixel;
                if i + state.bytes_per_pixel > state.backbuffer.len() { continue; }

                state.backbuffer.as_mut_slice()[i]     = color.b;
                state.backbuffer.as_mut_slice()[i + 1] = color.g;
                state.backbuffer.as_mut_slice()[i + 2] = color.r;
                if state.bytes_per_pixel == 4 {
                    state.backbuffer.as_mut_slice()[i + 3] = 0x00;
                }
            }
        }

        current_x += font_width;
    }
}

pub fn image_rgba_draw(x: usize, y: usize, width: usize, height: usize, data: &[u8]) {
    let mut guard = FB_STATE.lock(); // un seul lock
    let state = match guard.as_mut() {
        Some(s) => s,
        None => return,
    };
    
    for py in 0..height {
        for px in 0..width {
            let src = (py * width + px) * 4;
            if src + 3 >= data.len() { continue; }

            let a = data[src + 3];
            if a == 0 { continue; }

            let sx = x + px;
            let sy = y + py;
            if sx >= state.width || sy >= state.height { continue; }

            let i = sy * state.stride_bytes + sx * state.bytes_per_pixel;
            if i + state.bytes_per_pixel > state.backbuffer.len() { continue; }

            state.backbuffer.as_mut_slice()[i]     = data[src + 2]; // b
            state.backbuffer.as_mut_slice()[i + 1] = data[src + 1]; // g
            state.backbuffer.as_mut_slice()[i + 2] = data[src];     // r
            if state.bytes_per_pixel == 4 {
                state.backbuffer.as_mut_slice()[i + 3] = 0x00;
            }
        }
    }
}

pub fn capture_area(x: usize, y: usize, width: usize, height: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(width * height * 4);

    if let Some(ref state) = *FB_STATE.lock() {
        for py in 0..height {
            for px in 0..width {
                let sx = x + px;
                let sy = y + py;

                if sx >= state.width || sy >= state.height {
                    result.extend_from_slice(&[0, 0, 0, 0]);
                    continue;
                }

                let i =
                    sy * state.stride_bytes +
                    sx * state.bytes_per_pixel;

                if i + 3 >= state.backbuffer.as_slice().len() {
                    result.extend_from_slice(&[0, 0, 0, 0]);
                    continue;
                }

                let b = state.backbuffer.as_slice()[i];
                let g = if state.bytes_per_pixel > 1 { state.backbuffer.as_slice()[i + 1] } else { 0 };
                let r = if state.bytes_per_pixel > 2 { state.backbuffer.as_slice()[i + 2] } else { 0 };

                result.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }

    result
}



pub fn swap_buffers() {
    if let Some(ref mut state) = *FB_STATE.lock() {
        state.buffer.copy_from_slice(state.backbuffer.as_slice());
    }
}

pub fn buffer_image_rgba_draw(x: usize, y: usize, width: usize, height: usize, data: &[u8]) {
    let mut guard = FB_STATE.lock();
    let state = match guard.as_mut() {
        Some(s) => s,
        None => return,
    };

    for py in 0..height {
        let sy = y + py;
        if sy >= state.height { continue; }
        
        let row_start = sy * state.stride_bytes;

        for px in 0..width {
            let sx = x + px;
            if sx >= state.width { continue; }

            let src_idx = (py * width + px) * 4;

            if src_idx + 3 >= data.len() {
                continue;
            }

            let r = data[src_idx];
            let g = data[src_idx + 1];
            let b = data[src_idx + 2];
            let a = data[src_idx + 3];

            if a > 0 {
                let dest_idx = row_start + sx * state.bytes_per_pixel;

                if dest_idx + state.bytes_per_pixel <= state.buffer.len() {
                    state.buffer[dest_idx] = b;
                    if state.bytes_per_pixel > 1 {
                        state.buffer[dest_idx + 1] = g;
                    }
                    if state.bytes_per_pixel > 2 {
                        state.buffer[dest_idx + 2] = r;
                    }
                    if state.bytes_per_pixel > 3 {
                        state.buffer[dest_idx + 3] = 0x00;
                    }
                }
            }
        }
    }
}
