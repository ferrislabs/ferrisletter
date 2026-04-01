use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_src = Path::new(&manifest_dir).join("../../ui/dist/index.html");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("ui_bundle.html");

    if ui_src.exists() {
        fs::copy(&ui_src, &dest).expect("failed to copy ui bundle");
        println!("cargo:rerun-if-changed=../../ui/dist/index.html");
    } else {
        // Placeholder so include_str! compiles without a pre-built UI.
        fs::write(
            &dest,
            "<!DOCTYPE html><html><head><title>Ferrisletter</title></head>\
             <body style=\"font-family:sans-serif;padding:2rem\">\
             <h2>Ferrisletter UI not built</h2>\
             <p>Run <code>npm run build</code> inside the <code>ui/</code> directory, \
             then rebuild the server binary.</p></body></html>",
        )
        .expect("failed to write placeholder");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
