use crate::{apps::AppInfo};

// static mut KERNEL_API: Option<KernelApi> = None;

// const DOOM_BASE:          usize = 0x0400_0000;
// const DOOM_START_OFFSET:  usize = 0x350;
// const DOOM_TICK_OFFSET:   usize = 0x460;
// const DOOM_BSS_START:     usize = 0x06478338;
// const DOOM_BSS_END:       usize = 0x064b5d00 + 0x10000;

// static mut DOOM_STACK: [u8; 512 * 1024] = [0; 512 * 1024];

pub async fn doom(_app_info: AppInfo) {
    // {
    //     let mut manager = frame_buffer::LAYER_MANAGER.lock();
    //     if let Some(app_layer_id) = app_info.layer_id {
    //         if let Some(layer) = manager.get_layer_mut(app_layer_id) {
    //             layer.put_pixel(5, 25, ColorRGB::new(0x00,0x00,0x00));

    //             layer.text_draw(
    //                 (app_info.window_width - 12 * 32) / 2,
    //                 (app_info.window_height - 64) / 2,
    //                 "COMMING SOON", 
    //                 FontName::SpleenBigBig, 
    //                 ColorRGB::new(0x0a, 0x0a, 0x0a),
    //                 ColorRGB::new(0xd9, 0xd9, 0xd9),
    //             );
    //         }
    //     }
    // }



    // let Some(bytes) = boot_info::load_file("doom.bin") else {
    //     serial_println!("Doom: failed to load doom.bin");
    //     return;
    // };

    
    // unsafe {
    //     // 1. Copier le binary
    //     core::ptr::copy_nonoverlapping(
    //         bytes.as_ptr(),
    //         DOOM_BASE as *mut u8,
    //         bytes.len(),
    //     );

    //     // 2. Zero le BSS
    //     core::ptr::write_bytes(
    //         DOOM_BSS_START as *mut u8,
    //         0,
    //         DOOM_BSS_END - DOOM_BSS_START,
    //     );

    //     // 3. Stocker l'API
    //     KERNEL_API = Some(KernelApi {
    //         draw_frame: doom_update_screen,
    //         get_key:    doom_get_key,
    //         get_ticks:  doom_get_ticks,
    //     });
    //     let api_ptr = KERNEL_API.as_ref().unwrap() as *const KernelApi;

    //     let stack_top = (core::ptr::addr_of!(DOOM_STACK) as usize 
    //         + core::mem::size_of_val(&DOOM_STACK)) & !0xF;

    //     let doom_start_addr = DOOM_BASE + DOOM_START_OFFSET;

    //     serial_println!("Doom: calling _start on dedicated stack");

    //     // Switch de stack, appel, puis restore la stack originale
    //     core::arch::asm!(
    //         "mov {old_rsp}, rsp",   // sauvegarde rsp original
    //         "mov rsp, {stack}",     // switch vers stack doom
    //         "call {func}",          // appelle _start(api_ptr)
    //         "mov rsp, {old_rsp}",   // restore rsp original
    //         old_rsp = out(reg) _,
    //         stack = in(reg) stack_top,
    //         func = in(reg) doom_start_addr,
    //         in("rdi") api_ptr,
    //         options(nostack)
    //     );

    //     serial_println!("Doom: _start returned");
    // }

    // // 5. Tick loop async — aussi sur la stack doom
    // let doom_tick_addr = DOOM_BASE + DOOM_TICK_OFFSET;

    // loop {
    //     unsafe {
    //         let stack_top = (core::ptr::addr_of!(DOOM_STACK) as usize 
    //             + core::mem::size_of_val(&DOOM_STACK)) & !0xF;

    //         core::arch::asm!(
    //             "mov {old_rsp}, rsp",
    //             "mov rsp, {stack}",
    //             "call {func}",
    //             "mov rsp, {old_rsp}",
    //             old_rsp = out(reg) _,
    //             stack = in(reg) stack_top,
    //             func = in(reg) doom_tick_addr,
    //             options(nostack)
    //         );
    //     }
    //     crate::task::sleep::sleep_ms(1000 / 35).await;
    // }
}


// #[repr(C)]
// pub struct KernelApi {
//     pub draw_frame: extern "C" fn(pixels: *const u32, width: u32, height: u32),
//     pub get_key:    extern "C" fn(pressed: *mut i32, key: *mut u8) -> i32,
//     pub get_ticks:  extern "C" fn() -> u32,
// }

// extern "C" fn doom_update_screen(buffer: *const u32, width: u32, height: u32) {
//     serial_println!("doom_update_screen: {}x{} buf={:?}", width, height, buffer);
//     if buffer.is_null() { return; }

//     let w = width as usize;
//     let h = height as usize;
//     let total_bytes = w * h * 4;
//     let mut rgba_vec = alloc::vec![0u8; total_bytes];

//     unsafe {
//         core::ptr::copy_nonoverlapping(buffer as *const u8, rgba_vec.as_mut_ptr(), total_bytes);
//         if let Some(layer_id) = DOOM_LAYER_ID {
//             let mut manager = frame_buffer::LAYER_MANAGER.lock();
//             if let Some(layer) = manager.get_layer_mut(layer_id) {
//                 layer.image_rgba_draw(0, 0, w, h, rgba_vec.as_slice());
//             }
//         }
//     }
// }

// extern "C" fn doom_get_key(pressed: *mut i32, key: *mut u8) -> i32 {
//     unsafe {
//         if !pressed.is_null() { *pressed = 0; }
//         if !key.is_null()     { *key = 0; }
//     }
//     0
// }

// extern "C" fn doom_get_ticks() -> u32 {
//     // TICKS est en millisecondes si ton PIT est à 1000Hz, sinon adapte
//     crate::task::TICKS.load(core::sync::atomic::Ordering::Relaxed) as u32
// }