use std::{env, fs, io::Write, mem};


#[repr(C, packed)]
struct ManifestHeader {
    magic: [u8; 4],
    version: u32,
    kernel_sectors: u32,
    file_count: u32,
    entry_size: u32,
}

#[repr(C, packed)]
struct FileEntry {
    name: [u8; 32],
    start_sector: u32,
    sector_count: u32,
    file_size: u64,
    flags: u32,
    reserved: u32,
}



fn sectors_for(size: usize) -> u32 {
    ((size + 511) / 512) as u32
}

fn generate_kernel_sectors_inc() {
    let kernel = fs::metadata("kernel.bin").unwrap();
    let kernel_size = kernel.len() as usize;
    let kernel_sectors = sectors_for(kernel_size);
    let first = kernel_sectors.min(127);
    let second = kernel_sectors - first;

    let text = format!(
        "%define KERNEL_SECTORS {}\n\
         %define KERNEL_FIRST_READ_SECTORS {}\n\
         %define KERNEL_SECOND_READ_SECTORS {}\n",
        kernel_sectors,
        first,
        second
    );

    fs::write(
        "bootloader/kernel_sectors.inc",
        text,
    )
    .unwrap();
}

fn build_image() {
    let boot = fs::read("bootloader/boot.bin").unwrap();
    let stage2 = fs::read("bootloader/stage2.bin").unwrap();
    let kernel = fs::read("kernel.bin").unwrap();
    let payload = fs::read("kernel/resources/bad_apple.bin").unwrap();

    let kernel_sectors = sectors_for(kernel.len());
    let payload_sectors = sectors_for(payload.len());
    let payload_start = 5 + kernel_sectors;

    let mut name = [0u8; 32];
    let file_name = b"bad_apple.bin";
    name[..file_name.len()].copy_from_slice(file_name);

    let header = ManifestHeader {
        magic: *b"OSMF",
        version: 1,
        kernel_sectors,
        file_count: 1,
        entry_size: mem::size_of::<FileEntry>() as u32, 
    };

    let entry = FileEntry {
        name,
        start_sector: payload_start,
        sector_count: payload_sectors,
        file_size: payload.len() as u64,
        flags: 1,
        reserved: 0,
    };

    let image_size = ((payload_start + payload_sectors) * 512) as usize;
    let mut img = vec![0u8; image_size];

    // boot sector
    img[0..boot.len()].copy_from_slice(&boot);

    // stage2
    img[512..512 + stage2.len()].copy_from_slice(&stage2);

    // manifest
    let header_bytes = unsafe {
        core::slice::from_raw_parts(
            (&header as *const ManifestHeader) as *const u8,
            mem::size_of::<ManifestHeader>(),
        )
    };

    let entry_bytes = unsafe {
        core::slice::from_raw_parts(
            (&entry as *const FileEntry) as *const u8,
            mem::size_of::<FileEntry>(),
        )
    };

    let manifest_offset = 2048;
    img[manifest_offset..manifest_offset + header_bytes.len()].copy_from_slice(header_bytes);
    
    let entry_offset = manifest_offset + header_bytes.len();
    img[entry_offset..entry_offset + entry_bytes.len()].copy_from_slice(entry_bytes);

    let mut padded_manifest = [0u8; 512];
    padded_manifest[..header_bytes.len()].copy_from_slice(header_bytes);
    padded_manifest[header_bytes.len()..header_bytes.len() + entry_bytes.len()].copy_from_slice(entry_bytes);

    fs::write(
        "bootloader/boot_manifest.bin",
        padded_manifest,
    ).unwrap();

    // kernel @ LBA 5
    let kernel_offset = 5 * 512;
    img[kernel_offset..kernel_offset + kernel.len()].copy_from_slice(&kernel);

    // payload
    let payload_offset = payload_start as usize * 512;
    img[payload_offset..payload_offset + payload.len()].copy_from_slice(&payload);

    let mut file = fs::File::create("os.img").unwrap();
    file.write_all(&img).unwrap();

    println!("Built os.img");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "kernel-sectors" => {
                generate_kernel_sectors_inc();
            }

            _ => {
                panic!("unknown command");
            }
        }

    } else {
        build_image();
    }
}