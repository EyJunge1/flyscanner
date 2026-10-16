//! Ensure `frontend/build` exists so `rust-embed` can compile without a prior npm build.
//! Real assets: `make web` / `npm run build` before `cargo build`.

use std::fs;
use std::path::Path;

fn main() {
    let build = Path::new("frontend/build");
    let index = build.join("index.html");

    if !index.exists() {
        let _ = fs::create_dir_all(build);
        let _ = fs::write(
            &index,
            concat!(
                "<!doctype html><html><head><meta charset=utf-8>",
                "<title>flyscanner</title></head><body>",
                "<p>UI not embedded. Run <code>cd frontend && npm run build</code> then rebuild.</p>",
                "</body></html>\n"
            ),
        );
        println!(
            "cargo:warning=frontend/build missing — stub index.html embedded (run `make web`)"
        );
    }

    println!("cargo:rerun-if-changed=frontend/build");
    println!("cargo:rerun-if-changed=frontend/build/index.html");
}
