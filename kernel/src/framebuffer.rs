use core::{ptr::null_mut, slice};
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

unsafe impl Send for FrameBuffer {}
unsafe impl Sync for FrameBuffer {}


struct FrameBufferState {
    buffer: &'static mut [u8],
    stride_bytes: usize,
    width: usize,
    height: usize,
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
    let fb_addr = *(vbe_info.add(40) as *const u32);
    let fb_ptr = fb_addr as *mut u8;

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