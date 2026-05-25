use core::slice;
use spin::Mutex;


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

struct FrameBufferState {
    buffer: &'static mut [u8],
    stride_bytes: usize,
    width: usize,
    height: usize,
}

static FB_STATE: Mutex<Option<FrameBufferState>> = Mutex::new(None);

pub fn init(framebuffer: &'static FrameBuffer) {
    let buffer_slice = unsafe {
        slice::from_raw_parts_mut(framebuffer.buffer_ptr, framebuffer.buffer_size)
    };

    let mut guard = FB_STATE.lock();
    *guard = Some(FrameBufferState {
        buffer: buffer_slice,
        stride_bytes: framebuffer.stride,
        width: framebuffer.width,
        height: framebuffer.height,
    });
}

pub fn put_pixel(x: usize, y: usize, color: ColorRGB) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        let i = y * state.stride_bytes + x * 3;

        if i + 3 < state.buffer.len() {
            state.buffer[i]     = color.b;
            state.buffer[i + 1] = color.g;
            state.buffer[i + 2] = color.r;
        }
    }
}

pub fn clear(color: ColorRGB) {
    if let Some(ref mut state) = *FB_STATE.lock() {

        for y in 0..state.height {
            let row = y * state.stride_bytes;

            for x in 0..state.width {
                let i = row + x * 3;

                state.buffer[i]     = color.b;
                state.buffer[i + 1] = color.g;
                state.buffer[i + 2] = color.r;
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