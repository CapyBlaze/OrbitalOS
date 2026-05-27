use core::{ptr::null_mut, slice, str};
use alloc::{vec::Vec};
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
    pub fn new(r: u8, g: u8, b: u8) -> Self {
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
    Unknown,
}

impl FontName {
    pub fn as_str(&self) -> &'static str {
        match self {
            FontName::SpleenSmallSmall => "Spleen Small Small",
            FontName::SpleenSmall => "Spleen Small",
            FontName::SpleenLarge => "Spleen Large",
            FontName::SpleenBig => "Spleen Big",
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

    let mut guard = FB_STATE.lock();
    *guard = Some(FrameBufferState {
        buffer: buffer_slice,
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
}

pub fn put_pixel(x: usize, y: usize, color: ColorRGB) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        let i = y * state.stride_bytes + x * state.bytes_per_pixel;

        if i + state.bytes_per_pixel <= state.buffer.len() {
            state.buffer[i] = color.b;
            if state.bytes_per_pixel > 1 {
                state.buffer[i + 1] = color.g;
            }
            if state.bytes_per_pixel > 2 {
                state.buffer[i + 2] = color.r;
            }
            if state.bytes_per_pixel > 3 {
                state.buffer[i + 3] = 0x00;
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

            if i + 3 >= state.buffer.len() {
                continue;
            }

            state.buffer[i + 0] = color.b;
            state.buffer[i + 1] = color.g;
            state.buffer[i + 2] = color.r;

            if state.bytes_per_pixel == 4 {
                state.buffer[i + 3] = 0;
            }
        }
    }
}

pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: ColorRGB) {
    for py in 0..height {
        for px in 0..width {
            put_pixel(x + px, y + py, color);
        }
    }
}

pub fn clear(color: ColorRGB) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        for y in 0..state.height {
            let row = y * state.stride_bytes;

            for x in 0..state.width {
                let i = row + x * state.bytes_per_pixel;

                state.buffer[i] = color.b;
                if state.bytes_per_pixel > 1 {
                    state.buffer[i + 1] = color.g;
                }
                if state.bytes_per_pixel > 2 {
                    state.buffer[i + 2] = color.r;
                }
                if state.bytes_per_pixel > 3 {
                    state.buffer[i + 3] = 0x00;
                }
            }
        }
    }
}

pub fn text_draw(x: usize, y: usize, text: &str, font_name: FontName, fg: ColorRGB, bg: ColorRGB) {
    let manager = FONT_MANAGER.lock();
    let Some(font) = manager.iter().find(|f| f.name == font_name) else {
        return;
    };

    let mut current_x = x;

    for c in text.chars() {
        let ascii = c as usize;
        if ascii < 32 { continue; }

        let bitmap_start = 52 + (ascii - 32) * font.char_size;

        let glyph_end = bitmap_start + font.char_size;
        let Some(glyph_bytes) = font.data.get(bitmap_start..glyph_end) else {
            continue;
        };

        draw_bitmap_1bpp(
            current_x,
            y,
            font.font_bounding_box.0 as usize,
            font.font_bounding_box.1 as usize,
            glyph_bytes,
            fg,
            bg,
        );

        current_x += font.font_bounding_box.0 as usize;
    }
}