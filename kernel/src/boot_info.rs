use alloc::{vec, vec::Vec};
use core::mem::size_of;

use crate::{drivers::ata, generated::layout::MANIFEST_SECTOR};


#[repr(C)]
pub struct BootManifest {
    pub magic: [u8; 4],
    pub version: u32,
    pub kernel_sectors: u32,
    pub file_count: u32,
    pub entry_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BootFileEntry {
    pub name: [u8; 32],
    pub start_sector: u32,
    pub sector_count: u32,
    pub size: u64,
    pub flags: u32,
    pub reserved: u32,
}

impl BootFileEntry {
    pub fn name_as_str(&self) -> &str {
        let len = self
            .name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.name.len());

        core::str::from_utf8(&self.name[..len]).unwrap_or("")
    }
}

pub unsafe fn init(_manifest: *const BootManifest) {}

fn read_manifest() -> [u8; 512] {
    let mut sector = [0u8; 512];
    ata::read_sectors(MANIFEST_SECTOR, 1, &mut sector);

    sector
}

pub fn find_file(name: &str) -> Option<BootFileEntry> {
    let sector = read_manifest();
    let manifest = unsafe { &*(sector.as_ptr() as *const BootManifest) };
    assert_eq!(manifest.magic, *b"OSMF", "invalid boot manifest magic");
    assert_eq!(manifest.entry_size as usize, size_of::<BootFileEntry>());

    let entries_ptr = unsafe {
        sector.as_ptr().add(size_of::<BootManifest>()) as *const BootFileEntry
    };
    let entries = unsafe {
        core::slice::from_raw_parts(entries_ptr, manifest.file_count as usize)
    };

    entries
        .iter()
        .find(|entry| entry.name_as_str() == name)
        .copied()
}

pub fn load_file(name: &str) -> Option<Vec<u8>> {
    let entry = find_file(name)?;
    let sector_buffer_len = entry.sector_count as usize * 512;
    let mut buffer = vec![0u8; sector_buffer_len];

    if !buffer.is_empty() {
        ata::read_sectors(entry.start_sector, entry.sector_count, buffer.as_mut_slice());
    }

    buffer.truncate(entry.size as usize);

    Some(buffer)
}
