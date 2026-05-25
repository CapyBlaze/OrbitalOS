#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use os::allocator::HEAP_SIZE;
use os::memory::MemoryRegion;
use core::panic::PanicInfo;
use alloc::boxed::Box;
use alloc::vec::Vec;


#[no_mangle]
pub extern "C" fn _start(
    _vbe_info: *const u32,
    memory_map: *const MemoryRegion,
    memory_map_len: usize,
) -> ! {
    use os::allocator;
    use os::memory;

    os::init();
    

    let mut mapper = unsafe {
        memory::init()
    };

    let memory_regions = unsafe {
        core::slice::from_raw_parts(memory_map, memory_map_len)
    };

    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(memory_regions)
    };
    

    
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();
    
    os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}



#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}