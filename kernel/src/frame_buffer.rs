use core::{ptr::null_mut, slice};
use spin::Mutex;

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

pub fn draw_bitmap_1bpp(
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

    for py in 0..height {
        for px in 0..width {
            let bit_index = py * width + px;

            let byte = bitmap[bit_index / 8];
            let bit = (byte >> (7 - (bit_index % 8))) & 1;

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

pub fn draw_test() {
    for x in 0..200 {
        for y in 0..200 {
            put_pixel(x, y, ColorRGB::new(0xFF, 0x00, 0x00)); // rouge
        }
    }
}
