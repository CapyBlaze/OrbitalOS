#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[repr(C)]
pub struct KernelApi {
    pub draw_frame: extern "C" fn(pixels: *const u32, width: u32, height: u32),
    pub get_key:    extern "C" fn(pressed: *mut i32, key: *mut u8) -> i32,
    pub get_ticks:  extern "C" fn() -> u32,
}

unsafe extern "C" {
    fn doom_os_init(api: *const KernelApi);
    fn doom_os_tick();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(api: *const KernelApi) {
    unsafe { doom_os_init(api); }
}

#[unsafe(no_mangle)]
pub extern "C" fn doom_tick() {
    unsafe { doom_os_tick(); }
}

// malloc/free/memcpy/memset restent nécessaires pour le C
const DOOM_HEAP_SIZE: usize = 32 * 1024 * 1024;
static mut DOOM_HEAP: [u8; DOOM_HEAP_SIZE] = [0; DOOM_HEAP_SIZE];
static mut HEAP_INDEX: usize = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
    let align = 16;
    let size_aligned = (size + align - 1) & !(align - 1);
    unsafe {
        if HEAP_INDEX + size_aligned > DOOM_HEAP_SIZE { return core::ptr::null_mut(); }
        let ptr = (core::ptr::addr_of_mut!(DOOM_HEAP) as *mut u8).add(HEAP_INDEX);
        HEAP_INDEX += size_aligned;
        ptr as *mut core::ffi::c_void
    }
}

#[unsafe(no_mangle)] pub extern "C" fn free(_ptr: *mut u8) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut core::ffi::c_void {
    let total = nmemb * size;
    unsafe {
        let ptr = malloc(total);
        if !ptr.is_null() { core::ptr::write_bytes(ptr as *mut u8, 0, total); }
        ptr
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void {
    unsafe {
        let new_ptr = malloc(size);
        if !ptr.is_null() && !new_ptr.is_null() {
            core::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, size);
        }
        new_ptr
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dest, n); dest
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        core::ptr::write_bytes(s, c as u8, n); s
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fopen(_f: *const u8, _m: *const u8) -> *mut u8 { 0x1234 as *mut u8 }

static WAD_DATA: &[u8] = include_bytes!("../DOOM1.WAD");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fread(ptr: *mut u8, size: usize, nmemb: usize, _stream: *mut u8) -> usize {
    static mut WAD_OFFSET: usize = 0;
    let total = size * nmemb;
    unsafe {
        if WAD_OFFSET + total > WAD_DATA.len() { return 0; }
        core::ptr::copy_nonoverlapping(WAD_DATA.as_ptr().add(WAD_OFFSET), ptr, total);
        WAD_OFFSET += total;
    }
    nmemb
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }