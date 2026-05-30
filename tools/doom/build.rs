use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=src/doomgeneric/");

    let mut build = cc::Build::new();

    build
        .define("DOOMGENERIC", None)
        .define("NONET", None)
        .include("src/doomgeneric/")
        .warnings(false)
        .flag("-w");

    
    let paths = fs::read_dir("src/doomgeneric")
        .expect("Error: Directory not found!");


    let mut core_compiled = false;

    for path in paths {
        let entry = path.unwrap();
        let file_path = entry.path();
        
        if file_path.extension().and_then(|s| s.to_str()) == Some("c") {
            let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap();

            if file_name.starts_with("doomgeneric") {
                core_compiled = true;
            }

            if file_name.contains("allegro") {
                continue;
            }
            
            if file_name.contains("sdl") {
                continue;
            }

            if file_name == "doomgeneric_allegro.c" || 
                file_name == "doomgeneric_sdl.c" || 
                file_name == "doomgeneric_win32.c" || 
                file_name == "doomgeneric_win.c" || 
                file_name == "doomgeneric_x11.c" || 
                file_name == "doomgeneric_xlib.c" || 
                file_name == "doomgeneric_emscripten.c" || 
                file_name == "doomgeneric_sosox.c" || 
                file_name == "doomgeneric_soso.c" || 
                file_name == "doomgeneric_linuxvt.c" || 
                file_name == "i_main.c" || 
                file_name == "dummy.c" 
            {
                continue;
            }

            build.file(file_path);
        }
    }

    if !core_compiled {
        println!("cargo:warning=[DOOM-BUILD] Warning : No core DOOM files were compiled! Check the build script.");
    }

    build.define("DOOMGENERIC_RESX", Some("320"));
    build.define("DOOMGENERIC_RESY", Some("200"));

    // build.file("src/stubs.c");
    build.file("src/doomgeneric_os.c");
    
    build.compile("doomgeneric");
}