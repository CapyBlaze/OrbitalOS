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
    });
}

pub fn put_pixel(x: usize, y: usize, color: [u8; 4]) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        let i = y * state.stride_bytes + x * 4;

        if i + 3 < state.buffer.len() {
            state.buffer[i]     = color[0];
            state.buffer[i + 1] = color[1];
            state.buffer[i + 2] = color[2];
            state.buffer[i + 3] = color[3];
        }
    }
}

pub fn clear(color: [u8; 4]) {
    if let Some(ref mut state) = *FB_STATE.lock() {
        for i in (0..state.buffer.len()).step_by(4) {
            state.buffer[i]     = color[0];
            state.buffer[i + 1] = color[1];
            state.buffer[i + 2] = color[2];
            state.buffer[i + 3] = color[3];
        }
    }
}

pub fn draw_test() {
    for x in 0..200 {
        for y in 0..200 {
            put_pixel(x, y, [255, 0, 0, 255]); // rouge
        }
    }
}