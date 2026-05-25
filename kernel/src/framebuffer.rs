use core::slice;
use spin::Mutex;

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

pub fn put_pixel(x: usize, y: usize, color: u16) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        let pitch_pixels = state.stride_bytes / 2;

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                state.buffer.as_mut_ptr() as *mut u16,
                state.buffer.len() / 2,
            )
        };

        let i = y * pitch_pixels + x;

        if i < buffer.len() {
            buffer[i] = color;
        }
    }
}

pub fn clear(color: u16) {
    if let Some(ref mut state) = *FB_STATE.lock() {

        let pitch_pixels = state.stride_bytes / 2;

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(
                state.buffer.as_mut_ptr() as *mut u16,
                state.buffer.len() / 2,
            )
        };

        for y in 0..state.height {
            let row = y * pitch_pixels;

            for x in 0..state.width {
                buffer[row + x] = color;
            }
        }
    }
}

pub fn draw_test() {
    for x in 0..200 {
        for y in 0..200 {
            put_pixel(x, y, 0xF800); // rouge
        }
    }
}