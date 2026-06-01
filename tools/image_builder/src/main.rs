use std::{env, fs, io::Write, mem, path::Path};

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
    let second = kernel_sectors.saturating_sub(first);

    let text = format!(
        "%define KERNEL_SECTORS {}\n\
         %define KERNEL_FIRST_READ_SECTORS {}\n\
         %define KERNEL_SECOND_READ_SECTORS {}\n",
        kernel_sectors,
        first,
        second
    );

    fs::write("bootloader/kernel_sectors.inc", text).unwrap();
}

fn generate_kernel_layout_rs(stage2_sectors: u32) {
    let manifest_sector = 1 + stage2_sectors;

    let text = format!(
        "pub const MANIFEST_SECTOR: u32 = {};\n",
        manifest_sector
    );

    fs::write(
        "kernel/src/generated/layout.rs",
        text,
    )
    .unwrap();
}

fn collect_bin_files(path: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            collect_bin_files(&path, files);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|e| e.to_str());

        if extension == Some("bin") {
            files.push(path);
        }
    }
}

fn generate_layout_inc(stage2_sectors: u32, manifest_sectors: u32, kernel_sectors: u32) {
    let manifest_sector = 1 + stage2_sectors;
    let kernel_sector = manifest_sector + manifest_sectors;
    let resources_sector = kernel_sector + kernel_sectors;

    let text = format!(
        "%define MANIFEST_SECTOR {}\n\
         %define MANIFEST_SECTORS {}\n\
         %define KERNEL_START_SECTOR {}\n\
         %define RESOURCES_START_SECTOR {}\n",
        manifest_sector,
        manifest_sectors,
        kernel_sector,
        resources_sector
    );

    fs::write(
        "bootloader/layout.inc",
        text,
    )
    .unwrap();
}



// Layout disque =======
// LBA 0 -> boot
// LBA 1.. -> stage2
// puis manifest
// puis kernel
// puis resources

fn build_iso() {
    let boot = fs::read("bootloader/boot.bin").unwrap();
    let stage2 = fs::read("bootloader/stage2.bin").unwrap();
    let kernel = fs::read("kernel.bin").unwrap();

    let kernel_sectors = sectors_for(kernel.len());
    let stage2_sectors = sectors_for(stage2.len());

    let resources_path = Path::new("kernel/resources");
    let mut resource_files: Vec<(String, Vec<u8>)> = Vec::new();

    let mut raw_paths = Vec::new();
    collect_bin_files(resources_path, &mut raw_paths);

    for path in raw_paths {
        if !path.is_file() {
            continue;
        }

        let filename = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let data = fs::read(&path).unwrap();
        resource_files.push((filename, data));
    }

    let manifest_sector = 1 + stage2_sectors;

    let header = ManifestHeader {
        magic: *b"OSMF",
        version: 1,
        kernel_sectors,
        file_count: resource_files.len() as u32,
        entry_size: mem::size_of::<FileEntry>() as u32,
    };

    let manifest_raw_size = mem::size_of::<ManifestHeader>() + (resource_files.len() * mem::size_of::<FileEntry>());
    let manifest_sectors = sectors_for(manifest_raw_size);
    let kernel_sector = manifest_sector + manifest_sectors;
    let resources_start_sector = kernel_sector + kernel_sectors;

    generate_layout_inc(stage2_sectors, manifest_sectors, kernel_sectors);
    generate_kernel_layout_rs(stage2_sectors);

    let mut entries: Vec<(FileEntry, Vec<u8>)> = Vec::new();
    let mut current_sector = resources_start_sector;

    for (filename, data) in resource_files {
        let sector_count = sectors_for(data.len());

        let mut name = [0u8; 32];
        let bytes = filename.as_bytes();
        let len = bytes.len().min(32);
        name[..len].copy_from_slice(&bytes[..len]);

        let entry = FileEntry {
            name,
            start_sector: current_sector,
            sector_count,
            file_size: data.len() as u64,
            flags: 1,
            reserved: 0,
        };

        println!(
            "Adding resource: {} @ LBA {} ({} sectors)",
            filename,
            current_sector,
            sector_count
        );

        entries.push((entry, data));
        current_sector += sector_count;
    }

    let image_size = (current_sector * 512) as usize;
    let mut img = vec![0u8; image_size];

    // Boot sector
    img[0..boot.len()].copy_from_slice(&boot);

    // Stage2
    let stage2_offset = 512;
    img[stage2_offset..stage2_offset + stage2.len()].copy_from_slice(&stage2);

    // Manifest
    let manifest_offset = manifest_sector as usize * 512;

    let header_bytes = unsafe {
        core::slice::from_raw_parts(
            (&header as *const ManifestHeader) as *const u8,
            mem::size_of::<ManifestHeader>(),
        )
    };

    img[manifest_offset..manifest_offset + header_bytes.len()]
        .copy_from_slice(header_bytes);

    let mut manifest_write_offset = manifest_offset + header_bytes.len();

    for (entry, _) in &entries {
        let entry_bytes = unsafe {
            core::slice::from_raw_parts(
                (entry as *const FileEntry) as *const u8,
                mem::size_of::<FileEntry>(),
            )
        };

        img[manifest_write_offset..manifest_write_offset + entry_bytes.len()]
            .copy_from_slice(entry_bytes);

        manifest_write_offset += entry_bytes.len();
    }

    // Manifest dump (variable size, rounded to sectors)
    let mut padded_manifest = vec![0u8; (manifest_sectors * 512) as usize];

    padded_manifest[..header_bytes.len()]
        .copy_from_slice(header_bytes);

    let mut padded_offset = header_bytes.len();

    for (entry, _) in &entries {
        let entry_bytes = unsafe {
            core::slice::from_raw_parts(
                (entry as *const FileEntry) as *const u8,
                mem::size_of::<FileEntry>(),
            )
        };

        padded_manifest[padded_offset..padded_offset + entry_bytes.len()]
            .copy_from_slice(entry_bytes);

        padded_offset += entry_bytes.len();
    }

    fs::write(
        "bootloader/boot_manifest.bin",
        padded_manifest,
    )
    .unwrap();

    // Kernel
    let kernel_offset = kernel_sector as usize * 512;

    img[kernel_offset..kernel_offset + kernel.len()]
        .copy_from_slice(&kernel);

    // Resources
    for (entry, data) in &entries {
        let offset = entry.start_sector as usize * 512;

        img[offset..offset + data.len()]
            .copy_from_slice(data);
    }


    
    // Save image
    let mut file = fs::File::create("os.iso").unwrap();

    file.write_all(&img).unwrap();
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
        build_iso();
    }
}
