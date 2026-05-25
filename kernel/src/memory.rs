use x86_64::{
    PhysAddr, VirtAddr, 
    structures::paging::{OffsetPageTable, PageTable, Page, PhysFrame, Mapper, Size4KiB, FrameAllocator}
};


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MemoryRegion {
    pub base: u64,
    pub length: u64,
    pub region_type: u32,
    pub acpi: u32,
}


pub struct BootInfoFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static [MemoryRegion]) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        const MIN_USABLE_ADDR: u64 = 0x200000; // Skip low memory used by bootloader/kernel
        const MAX_MAPPED_ADDR: u64 = 0x1_0000_0000; // Identity map covers 0..4GiB

        let regions = self.memory_map.iter();
        
        let usable_regions = regions
            .filter(|r| r.region_type == 1);

        let addr_ranges = usable_regions
            .filter_map(|r| {
                if r.base >= MAX_MAPPED_ADDR {
                    return None;
                }

                let start = r.base.max(MIN_USABLE_ADDR);
                let end = r.base.saturating_add(r.length).min(MAX_MAPPED_ADDR);
                if start >= end {
                    None
                } else {
                    Some(start..end)
                }
            });
            
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}


unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}


pub unsafe fn init() -> OffsetPageTable<'static> {
    unsafe {
        let offset = VirtAddr::new(0);
        let level_4_table = active_level_4_table(offset);
        OffsetPageTable::new(level_4_table, offset)
    }
}


unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}


pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}


fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}


pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}