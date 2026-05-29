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


pub struct Layer {
    pub id: u64,
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    pub z_index: usize,
}

impl Layer {
    pub fn new(id: u64, width: usize, height: usize, x: usize, y: usize, z_index: usize) -> Self {
        Self {
            id,
            data: vec![0; width * height * 4],
            width,
            height,
            x,
            y,
            z_index,
        }
    }

    pub fn clear(&mut self, color: ColorRGB) {
        for i in 0..self.width * self.height {
            self.data.as_mut_slice()[i * 4]     = color.r;
            self.data.as_mut_slice()[i * 4 + 1] = color.g;
            self.data.as_mut_slice()[i * 4 + 2] = color.b;
            self.data.as_mut_slice()[i * 4 + 3] = 0xff;
        }
    }

    pub fn clear_transparent(&mut self) {
        self.data.fill(0);
    }


    pub fn put_pixel(&mut self, x: usize, y: usize, color: ColorRGB) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let i = (y * self.width + x) * 4;

        if i + 3 < self.data.len() {
            self.data.as_mut_slice()[i]     = color.r;
            self.data.as_mut_slice()[i + 1] = color.g;
            self.data.as_mut_slice()[i + 2] = color.b;
            self.data.as_mut_slice()[i + 3] = 0xff;
        }
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: ColorRGB) {
        for py in 0..height {
            for px in 0..width {
                self.put_pixel(x + px, y + py, color);
            }
        }
    }

    pub fn draw_circle(&mut self, x: usize, y: usize, width: usize, height: usize, color: ColorRGB) {
        if width == 0 || height == 0 {
            return;
        }

        let rx = width as f64 / 2.0;
        let ry = height as f64 / 2.0;

        let cx = x as f64 + rx;
        let cy = y as f64 + ry;

        for py in 0..height {
            for px in 0..width {
                let pixel_x = (x + px) as f64 + 0.5;
                let pixel_y = (y + py) as f64 + 0.5;

                let dx = pixel_x - cx;
                let dy = pixel_y - cy;

                if (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0 {
                    self.put_pixel(x + px, y + py, color);
                }
            }
        }
    }


    
    pub fn draw_bitmap_1bpp(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        bitmap: &[u8],
        fg: ColorRGB,
        bg: ColorRGB,
    ) {
        let bytes_per_row = (width + 7) / 8;
        
        for py in 0..height {
            for px in 0..width {
                let byte_index = py * bytes_per_row + (px / 8);
                let byte = bitmap[byte_index];
                let bit = (byte >> (7 - (px % 8))) & 1;

                let color = if bit == 1 { fg } else { bg };
                self.put_pixel(x + px, y + py, color);
            }
        }
    }

    pub fn text_draw(&mut self, x: usize, y: usize, text: &str, font_name: FontName, fg: ColorRGB, bg: ColorRGB) {
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

                    self.put_pixel(current_x + px, y + py, color);
                }
            }
            current_x += font_width;
        }
    }

    pub fn image_rgba_draw(&mut self, x: usize, y: usize, width: usize, height: usize, data: &[u8]) {
        for py in 0..height {
            for px in 0..width {
                let src = (py * width + px) * 4;
                if src + 3 >= data.len() { continue; }

                let a = data[src + 3];
                if a == 0 { continue; }

                let dst = ((y + py) * self.width + (x + px)) * 4;
                if dst + 3 < self.data.len() {
                    self.data.as_mut_slice()[dst]     = data[src];     // R
                    self.data.as_mut_slice()[dst + 1] = data[src + 1]; // G
                    self.data.as_mut_slice()[dst + 2] = data[src + 2]; // B
                    self.data.as_mut_slice()[dst + 3] = a;             // A
                }
            }
        }
    }
}

pub struct LayerManager {
    pub layers: Vec<Layer>,
    next_id: u64,
}

impl LayerManager {
    pub const fn new() -> Self {
        Self {
            layers: Vec::new(),
            next_id: 1,
        }
    }

    pub fn create_layer(&mut self, width: usize, height: usize, x: usize, y: usize, z_index: usize) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let layer = Layer::new(id, width, height, x, y, z_index);
        self.push_layer(layer);

        id
    }

    pub fn get_layer_mut(&mut self, id: u64) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    pub fn remove_layer(&mut self, id: u64) {
        self.layers.retain(|l| l.id != id);
    }

    pub fn push_layer(&mut self, layer: Layer) -> usize {
        self.layers.push(layer);
        self.layers.len() - 1
    }

    pub fn insert_at(&mut self, z_index: usize, layer: Layer) {
        if z_index >= self.layers.len() {
            self.layers.push(layer);
        } else {
            self.layers.insert(z_index, layer);
        }
    }

    pub fn bring_to_front(&mut self, id: u64) {
        if let Some(index) = self.layers.iter().position(|l| l.id == id) {
            let layer = self.layers.remove(index);
            self.layers.push(layer);
        }
    }

    pub fn compose(&self, backbuffer: &mut [u8], screen_width: usize, screen_height: usize, stride_bytes: usize, bytes_per_pixel: usize) {
        let mut sorted_refs = [None; 64];
        let count = self.layers.len().min(64);
        
        for i in 0..count {
            sorted_refs[i] = Some(&self.layers.as_slice()[i]);
        }
        
        let slice = &mut sorted_refs[0..count];
        for i in 1..slice.len() {
            let mut j = i;
            while j > 0 && slice[j - 1].unwrap().z_index > slice[j].unwrap().z_index {
                slice.swap(j - 1, j);
                j -= 1;
            }
        }

        for layer_opt in slice {
            let layer = layer_opt.unwrap();
            for y in 0..layer.height {
                let screen_y = layer.y + y;
                if screen_y >= screen_height { continue; }

                for x in 0..layer.width {
                    let screen_x = layer.x + x;
                    if screen_x >= screen_width { continue; }

                    let src = (y * layer.width + x) * 4;
                    if src + 3 >= layer.data.len() { continue; }

                    let a = layer.data[src + 3];
                    if a == 0 { continue; }

                    let dst = screen_y * stride_bytes + screen_x * bytes_per_pixel;
                    if dst + bytes_per_pixel > backbuffer.len() { continue; }

                    let r = layer.data[src];
                    let g = layer.data[src + 1];
                    let b = layer.data[src + 2];

                    if a == 0xff {
                        backbuffer[dst]     = b;
                        backbuffer[dst + 1] = g;
                        backbuffer[dst + 2] = r;
                    } else {
                        let a16 = a as u16;
                        backbuffer[dst]     = ((b as u16 * a16 + backbuffer[dst]     as u16 * (255 - a16)) / 255) as u8;
                        backbuffer[dst + 1] = ((g as u16 * a16 + backbuffer[dst + 1] as u16 * (255 - a16)) / 255) as u8;
                        backbuffer[dst + 2] = ((r as u16 * a16 + backbuffer[dst + 2] as u16 * (255 - a16)) / 255) as u8;
                    }
                    if bytes_per_pixel == 4 { backbuffer[dst + 3] = 0; }
                }
            }
        }
    }
}

pub static LAYER_MANAGER: Mutex<LayerManager> = Mutex::new(LayerManager::new());



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


pub fn draw_layers_to_screen() {
    if let Some(ref mut state) = *FB_STATE.lock() {
        state.backbuffer.fill(0); 

        LAYER_MANAGER.lock().compose(
            &mut state.backbuffer.as_mut_slice(),
            state.width,
            state.height,
            state.stride_bytes,
            state.bytes_per_pixel,
        );

        state.buffer.copy_from_slice(state.backbuffer.as_slice());
    }
}
