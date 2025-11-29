use std::fs;
use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=icon.ico");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    
    // Navigate from OUT_DIR to the target directory
    // OUT_DIR is typically: target/debug/build/package-name/hash
    // We need to go up 3 levels to get to target/debug
    let target_dir = Path::new(&out_dir)
        .parent()      // build/package-name/hash
        .unwrap()
        .parent()      // build/package-name
        .unwrap()
        .parent()      // build
        .unwrap()
        .parent()      // target
        .unwrap()
        .join(&profile); // target/debug or target/release
    
    // Create target directory if it doesn't exist
    fs::create_dir_all(&target_dir).ok();
    
    // Copy icon.ico to the target directory
    let icon_source = Path::new("icon.ico");
    let icon_dest = target_dir.join("icon.ico");
    
    if icon_source.exists() {
        match fs::copy(icon_source, &icon_dest) {
            Ok(_) => println!("Copied icon.ico to {:?}", icon_dest),
            Err(e) => println!("Warning: Failed to copy icon.ico: {}", e),
        }
    } else {
        println!("Warning: icon.ico not found in project root");
    }
}
